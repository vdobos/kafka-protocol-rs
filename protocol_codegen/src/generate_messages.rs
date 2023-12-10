use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use failure::Error;
use git2::{Oid, Repository};

use std::path::MAIN_SEPARATOR;

mod code_writer;
pub mod expr;
mod generate;
mod parse;
mod spec;

use spec::SpecType;

pub fn run() -> Result<(), Error> {
    let mut dir = std::fs::canonicalize(std::file!().rsplit_once(MAIN_SEPARATOR).unwrap().0)?;
    dir.push(format!("..{separator}..{separator}src{separator}messages", separator = MAIN_SEPARATOR));
    let output_path = std::fs::canonicalize(dir)?;
    let output_path = output_path.to_str().unwrap();

    // Download messages from head of Kafka repo
    let kafka_repo = Path::new("kafka_repo");
    let repo = if kafka_repo.exists() {
        println!("Fetching latest kafka repo");
        let repo = Repository::open(kafka_repo)?;
        repo.find_remote("origin")
            .unwrap()
            .fetch(&["trunk"], None, None)
            .unwrap();
        repo
    } else {
        println!("Cloning kafka repo");
        git2::build::RepoBuilder::new()
            .fetch_options(git2::FetchOptions::new())
            .with_checkout(git2::build::CheckoutBuilder::new())
            .clone("https://github.com/apache/kafka.git", kafka_repo)?
    };

    // Checkout the release commit
    // https://github.com/apache/kafka/releases/tag/3.6.0
    // checking out a tag with git2 is annoying -- we pin to the tag's commit sha instead
    let release_commit = "60e845626d8a465a8cfe68bb2d7d4b88d622634e";
    println!("Checking out release {}", release_commit);
    let oid = Oid::from_str(release_commit).unwrap();
    let commit = repo
        .find_commit(oid)
        .expect("Could not find release commit!")
        .into_object();
    repo.checkout_tree(&commit, None).unwrap();
    repo.set_head_detached(commit.id()).unwrap();

    // Clear output directory
    for file in fs::read_dir(output_path)? {
        let file = file?;
        if file.file_type()?.is_file() {
            let path = file.path();
            if path.extension() == Some("rs".as_ref()) {
                fs::remove_file(path)?;
            }
        }
    }

    // Find input files
    let mut input_file_paths = Vec::new();
    for file in fs::read_dir(kafka_repo.join(format!("clients{separator}src{separator}main{separator}resources{separator}common{separator}message", separator = MAIN_SEPARATOR)))? {
        let file = file?;
        if file.file_type()?.is_file() {
            let path = file.path();
            if path.extension() == Some("json".as_ref()) {
                input_file_paths.push(path);
            }
        }
    }
    input_file_paths.sort();
    let mut entity_types = BTreeSet::new();
    let mut request_types = BTreeMap::new();
    let mut response_types = BTreeMap::new();

    let module_path = format!("{}.rs", output_path);
    let mut module_file = File::create(&module_path)?;

    writeln!(module_file, "//! Messages used by the Kafka protocol.")?;
    writeln!(module_file, "//!")?;
    writeln!(module_file, "//! These messages are generated programmatically. See the [Kafka's protocol documentation](https://kafka.apache.org/protocol.html) for more information about a given message type.")?;
    writeln!(
        module_file,
        "// WARNING: the items of this module are generated and should not be edited directly."
    )?;
    writeln!(module_file)?;
    writeln!(
        module_file,
        "use crate::protocol::{{NewType, Request, StrBytes, HeaderVersion}};"
    )?;
    writeln!(module_file, "use std::convert::TryFrom;")?;
    writeln!(module_file)?;

    for input_file_path in &input_file_paths {
        let spec = parse::parse(input_file_path)?;
        let spec_meta = (spec.type_, spec.api_key);
        let (module_name, struct_name) = generate::generate(output_path, spec, &mut entity_types)?;
		
        match spec_meta {
            (SpecType::Request, Some(k)) => {
                request_types.insert(k, struct_name.clone());
            }
            (SpecType::Response, Some(k)) => {
                response_types.insert(k, struct_name.clone());
            }
            _ => {}
        }

        writeln!(module_file, "pub mod {};", module_name)?;
        writeln!(module_file, "pub use {}::{};", module_name, struct_name)?;
        writeln!(module_file)?;
    }

    for (api_key, request_type) in request_types.iter() {
        let response_type = response_types
            .get(api_key)
            .expect("Every request type has a response type");
        writeln!(module_file, "impl Request for {} {{", request_type)?;
        writeln!(module_file, "    const KEY: i16 = {};", api_key)?;
        writeln!(module_file, "    type Response = {};", response_type)?;
        writeln!(module_file, "}}")?;
        writeln!(module_file)?;
    }

    writeln!(module_file, "/// Valid API keys in the Kafka protocol.")?;
    writeln!(module_file, "#[derive(Debug, Clone, Copy, PartialEq, Eq)]")?;
    writeln!(module_file, "pub enum ApiKey {{")?;
    for (api_key, request_type) in request_types.iter() {
        writeln!(module_file, "    /// API key for request {}", request_type)?;
        writeln!(
            module_file,
            "    {} = {},",
            request_type.replace("Request", "Key"),
            api_key
        )?;
    }
    writeln!(module_file, "}}")?;
    writeln!(module_file)?;

    writeln!(module_file, "impl ApiKey {{")?;
    writeln!(
        module_file,
        "    /// Get the version of request header that needs to be prepended to this message"
    )?;
    writeln!(
        module_file,
        "    pub fn request_header_version(&self, version: i16) -> i16 {{"
    )?;
    writeln!(module_file, "        match self {{")?;
    for request_type in request_types.values() {
        writeln!(
            module_file,
            "            ApiKey::{} => {}::header_version(version),",
            request_type.replace("Request", "Key"),
            request_type
        )?;
    }
    writeln!(module_file, "        }}")?;
    writeln!(module_file, "    }}")?;

    writeln!(
        module_file,
        "    /// Get the version of response header that needs to be prepended to this message"
    )?;
    writeln!(
        module_file,
        "    pub fn response_header_version(&self, version: i16) -> i16 {{"
    )?;
    writeln!(module_file, "        match self {{")?;
    for response_type in response_types.values() {
        writeln!(
            module_file,
            "            ApiKey::{} => {}::header_version(version),",
            response_type.replace("Response", "Key"),
            response_type
        )?;
    }
    writeln!(module_file, "        }}")?;
    writeln!(module_file, "    }}")?;
    writeln!(module_file, "}}")?;

    writeln!(module_file, "impl TryFrom<i16> for ApiKey {{")?;
    writeln!(module_file, "    type Error = ();")?;
    writeln!(module_file)?;
    writeln!(
        module_file,
        "    fn try_from(v: i16) -> Result<Self, Self::Error> {{"
    )?;
    writeln!(module_file, "        match v {{")?;
    for (_, request_type) in request_types.iter() {
        let key = request_type.replace("Request", "Key");
        writeln!(
            module_file,
            "            x if x == ApiKey::{} as i16 => Ok(ApiKey::{}),",
            key, key
        )?;
    }
    writeln!(module_file, "            _ => Err(()),")?;
    writeln!(module_file, "        }}")?;
    writeln!(module_file, "    }}")?;
    writeln!(module_file, "}}")?;
    writeln!(module_file)?;

    writeln!(
        module_file,
        "/// Wrapping enum for all requests in the Kafka protocol."
    )?;
    writeln!(module_file, "#[non_exhaustive]")?;
    writeln!(module_file, "#[derive(Debug, Clone, PartialEq)]")?;
    writeln!(module_file, "pub enum RequestKind {{")?;
    for (_, request_type) in request_types.iter() {
        writeln!(module_file, "    /// {},", request_type)?;
        writeln!(module_file, "    {}({}),", request_type, request_type)?;
    }
    writeln!(module_file, "}}")?;
    writeln!(module_file)?;

    writeln!(
        module_file,
        "/// Wrapping enum for all responses in the Kafka protocol."
    )?;
    writeln!(module_file, "#[non_exhaustive]")?;
    writeln!(module_file, "#[derive(Debug, Clone, PartialEq)]")?;
    writeln!(module_file, "pub enum ResponseKind {{")?;
    for (_, response_type) in response_types.iter() {
        writeln!(module_file, "    /// {},", response_type)?;
        writeln!(module_file, "    {}({}),", response_type, response_type)?;
    }
    writeln!(module_file, "}}")?;
    writeln!(module_file)?;

    for entity_type in entity_types {
        let mut derives = vec![
            "Debug",
            "Clone",
            "Eq",
            "PartialEq",
            "Ord",
            "PartialOrd",
            "Hash",
            "Default",
        ];
        if entity_type.inner.is_copy() {
            derives.push("Copy");
        }

        let rust_name = entity_type.inner.rust_name();

        writeln!(module_file, "/// {}", entity_type.doc)?;
        writeln!(module_file, "#[derive({})]", derives.join(", "))?;
        writeln!(
            module_file,
            "pub struct {}(pub {});\n",
            entity_type.name, rust_name
        )?;
        writeln!(
            module_file,
            "impl From<{}> for {} {{",
            rust_name, entity_type.name
        )?;
        writeln!(
            module_file,
            "    fn from(other: {}) -> Self {{ Self(other) }}",
            rust_name
        )?;
        writeln!(module_file, "}}")?;
        writeln!(
            module_file,
            "impl From<{}> for {} {{",
            entity_type.name, rust_name
        )?;
        writeln!(
            module_file,
            "    fn from(other: {}) -> Self {{ other.0 }}",
            entity_type.name
        )?;
        writeln!(module_file, "}}")?;
        writeln!(
            module_file,
            "impl std::borrow::Borrow<{}> for {} {{",
            rust_name, entity_type.name
        )?;
        writeln!(
            module_file,
            "    fn borrow(&self) -> &{} {{ &self.0 }}",
            rust_name
        )?;
        writeln!(module_file, "}}")?;
        writeln!(
            module_file,
            "impl std::ops::Deref for {} {{",
            entity_type.name
        )?;
        writeln!(module_file, "    type Target = {};", rust_name)?;
        writeln!(
            module_file,
            "    fn deref(&self) -> &Self::Target {{ &self.0 }}"
        )?;
        writeln!(module_file, "}}")?;
        writeln!(
            module_file,
            "impl std::cmp::PartialEq<{}> for {} {{",
            rust_name, entity_type.name
        )?;
        writeln!(
            module_file,
            "    fn eq(&self, other: &{}) -> bool {{ &self.0 == other }}",
            rust_name
        )?;
        writeln!(module_file, "}}")?;
        writeln!(
            module_file,
            "impl std::cmp::PartialEq<{}> for {} {{",
            entity_type.name, rust_name
        )?;
        writeln!(
            module_file,
            "    fn eq(&self, other: &{}) -> bool {{ self == &other.0 }}",
            entity_type.name
        )?;
        writeln!(module_file, "}}")?;
        writeln!(
            module_file,
            "impl NewType<{}> for {} {{}}",
            rust_name, entity_type.name
        )?;
        writeln!(module_file)?;
    }

    Ok(())
}
