//! ConsumerGroupHeartbeatRequest
//!
//! See the schema for this message [here](https://github.com/apache/kafka/blob/trunk/clients/src/main/resources/common/message/ConsumerGroupHeartbeatRequest.json).
// WARNING: the items of this module are generated and should not be edited directly
#![allow(unused)]

use std::borrow::Borrow;
use std::collections::BTreeMap;

use bytes::Bytes;
use log::error;
use uuid::Uuid;

use crate::protocol::{
    Encodable, Decodable, MapEncodable, MapDecodable, Encoder, Decoder, EncodeError, DecodeError, Message, HeaderVersion, VersionRange,
    types, write_unknown_tagged_fields, compute_unknown_tagged_fields_size, StrBytes, buf::{ByteBuf, ByteBufMut}, Builder
};


/// Valid versions: 0
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct Assignor {
    /// The name of the assignor.
    /// 
    /// Supported API versions: 0
    pub name: StrBytes,

    /// The minimum supported version for the metadata.
    /// 
    /// Supported API versions: 0
    pub minimum_version: i16,

    /// The maximum supported version for the metadata.
    /// 
    /// Supported API versions: 0
    pub maximum_version: i16,

    /// The reason of the metadata update.
    /// 
    /// Supported API versions: 0
    pub reason: i8,

    /// The version of the metadata.
    /// 
    /// Supported API versions: 0
    pub metadata_version: i16,

    /// The metadata.
    /// 
    /// Supported API versions: 0
    pub metadata_bytes: Bytes,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for Assignor {
    type Builder = AssignorBuilder;

    fn builder() -> Self::Builder{
        AssignorBuilder::default()
    }
}

impl Encodable for Assignor {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        types::CompactString.encode(buf, &self.name)?;
        types::Int16.encode(buf, &self.minimum_version)?;
        types::Int16.encode(buf, &self.maximum_version)?;
        types::Int8.encode(buf, &self.reason)?;
        types::Int16.encode(buf, &self.metadata_version)?;
        types::CompactBytes.encode(buf, &self.metadata_bytes)?;
        let num_tagged_fields = self.unknown_tagged_fields.len();
        if num_tagged_fields > std::u32::MAX as usize {
            error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
            return Err(EncodeError);
        }
        types::UnsignedVarInt.encode(buf, num_tagged_fields as u32)?;

        write_unknown_tagged_fields(buf, 0.., &self.unknown_tagged_fields)?;
        Ok(())
    }
    fn compute_size(&self, version: i16) -> Result<usize, EncodeError> {
        let mut total_size = 0;
        total_size += types::CompactString.compute_size(&self.name)?;
        total_size += types::Int16.compute_size(&self.minimum_version)?;
        total_size += types::Int16.compute_size(&self.maximum_version)?;
        total_size += types::Int8.compute_size(&self.reason)?;
        total_size += types::Int16.compute_size(&self.metadata_version)?;
        total_size += types::CompactBytes.compute_size(&self.metadata_bytes)?;
        let num_tagged_fields = self.unknown_tagged_fields.len();
        if num_tagged_fields > std::u32::MAX as usize {
            error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
            return Err(EncodeError);
        }
        total_size += types::UnsignedVarInt.compute_size(num_tagged_fields as u32)?;

        total_size += compute_unknown_tagged_fields_size(&self.unknown_tagged_fields)?;
        Ok(total_size)
    }
}

impl Decodable for Assignor {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let name = types::CompactString.decode(buf)?;
        let minimum_version = types::Int16.decode(buf)?;
        let maximum_version = types::Int16.decode(buf)?;
        let reason = types::Int8.decode(buf)?;
        let metadata_version = types::Int16.decode(buf)?;
        let metadata_bytes = types::CompactBytes.decode(buf)?;
        let mut unknown_tagged_fields = BTreeMap::new();
        let num_tagged_fields = types::UnsignedVarInt.decode(buf)?;
        for _ in 0..num_tagged_fields {
            let tag: u32 = types::UnsignedVarInt.decode(buf)?;
            let size: u32 = types::UnsignedVarInt.decode(buf)?;
            let mut unknown_value = vec![0; size as usize];
            buf.try_copy_to_slice(&mut unknown_value)?;
            unknown_tagged_fields.insert(tag as i32, unknown_value);
        }
        Ok(Self {
            name,
            minimum_version,
            maximum_version,
            reason,
            metadata_version,
            metadata_bytes,
            unknown_tagged_fields,
        })
    }
}

impl Default for Assignor {
    fn default() -> Self {
        Self {
            name: Default::default(),
            minimum_version: 0,
            maximum_version: 0,
            reason: 0,
            metadata_version: 0,
            metadata_bytes: Default::default(),
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for Assignor {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 0 };
}

/// Valid versions: 0
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct TopicPartitions {
    /// The topic ID.
    /// 
    /// Supported API versions: 0
    pub topic_id: Uuid,

    /// The partitions.
    /// 
    /// Supported API versions: 0
    pub partitions: Vec<i32>,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for TopicPartitions {
    type Builder = TopicPartitionsBuilder;

    fn builder() -> Self::Builder{
        TopicPartitionsBuilder::default()
    }
}

impl Encodable for TopicPartitions {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        types::Uuid.encode(buf, &self.topic_id)?;
        types::CompactArray(types::Int32).encode(buf, &self.partitions)?;
        let num_tagged_fields = self.unknown_tagged_fields.len();
        if num_tagged_fields > std::u32::MAX as usize {
            error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
            return Err(EncodeError);
        }
        types::UnsignedVarInt.encode(buf, num_tagged_fields as u32)?;

        write_unknown_tagged_fields(buf, 0.., &self.unknown_tagged_fields)?;
        Ok(())
    }
    fn compute_size(&self, version: i16) -> Result<usize, EncodeError> {
        let mut total_size = 0;
        total_size += types::Uuid.compute_size(&self.topic_id)?;
        total_size += types::CompactArray(types::Int32).compute_size(&self.partitions)?;
        let num_tagged_fields = self.unknown_tagged_fields.len();
        if num_tagged_fields > std::u32::MAX as usize {
            error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
            return Err(EncodeError);
        }
        total_size += types::UnsignedVarInt.compute_size(num_tagged_fields as u32)?;

        total_size += compute_unknown_tagged_fields_size(&self.unknown_tagged_fields)?;
        Ok(total_size)
    }
}

impl Decodable for TopicPartitions {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let topic_id = types::Uuid.decode(buf)?;
        let partitions = types::CompactArray(types::Int32).decode(buf)?;
        let mut unknown_tagged_fields = BTreeMap::new();
        let num_tagged_fields = types::UnsignedVarInt.decode(buf)?;
        for _ in 0..num_tagged_fields {
            let tag: u32 = types::UnsignedVarInt.decode(buf)?;
            let size: u32 = types::UnsignedVarInt.decode(buf)?;
            let mut unknown_value = vec![0; size as usize];
            buf.try_copy_to_slice(&mut unknown_value)?;
            unknown_tagged_fields.insert(tag as i32, unknown_value);
        }
        Ok(Self {
            topic_id,
            partitions,
            unknown_tagged_fields,
        })
    }
}

impl Default for TopicPartitions {
    fn default() -> Self {
        Self {
            topic_id: Uuid::nil(),
            partitions: Default::default(),
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for TopicPartitions {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 0 };
}

/// Valid versions: 0
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct ConsumerGroupHeartbeatRequest {
    /// The group identifier.
    /// 
    /// Supported API versions: 0
    pub group_id: super::GroupId,

    /// The member id generated by the coordinator. The member id must be kept during the entire lifetime of the member.
    /// 
    /// Supported API versions: 0
    pub member_id: StrBytes,

    /// The current member epoch; 0 to join the group; -1 to leave the group; -2 to indicate that the static member will rejoin.
    /// 
    /// Supported API versions: 0
    pub member_epoch: i32,

    /// null if not provided or if it didn't change since the last heartbeat; the instance Id otherwise.
    /// 
    /// Supported API versions: 0
    pub instance_id: Option<StrBytes>,

    /// null if not provided or if it didn't change since the last heartbeat; the rack ID of consumer otherwise.
    /// 
    /// Supported API versions: 0
    pub rack_id: Option<StrBytes>,

    /// -1 if it didn't chance since the last heartbeat; the maximum time in milliseconds that the coordinator will wait on the member to revoke its partitions otherwise.
    /// 
    /// Supported API versions: 0
    pub rebalance_timeout_ms: i32,

    /// null if it didn't change since the last heartbeat; the subscribed topic names otherwise.
    /// 
    /// Supported API versions: 0
    pub subscribed_topic_names: Option<Vec<super::TopicName>>,

    /// null if it didn't change since the last heartbeat; the subscribed topic regex otherwise
    /// 
    /// Supported API versions: 0
    pub subscribed_topic_regex: Option<StrBytes>,

    /// null if not used or if it didn't change since the last heartbeat; the server side assignor to use otherwise.
    /// 
    /// Supported API versions: 0
    pub server_assignor: Option<StrBytes>,

    /// null if not used or if it didn't change since the last heartbeat; the list of client-side assignors otherwise.
    /// 
    /// Supported API versions: 0
    pub client_assignors: Option<Vec<Assignor>>,

    /// null if it didn't change since the last heartbeat; the partitions owned by the member.
    /// 
    /// Supported API versions: 0
    pub topic_partitions: Option<Vec<TopicPartitions>>,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for ConsumerGroupHeartbeatRequest {
    type Builder = ConsumerGroupHeartbeatRequestBuilder;

    fn builder() -> Self::Builder{
        ConsumerGroupHeartbeatRequestBuilder::default()
    }
}

impl Encodable for ConsumerGroupHeartbeatRequest {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        types::CompactString.encode(buf, &self.group_id)?;
        types::CompactString.encode(buf, &self.member_id)?;
        types::Int32.encode(buf, &self.member_epoch)?;
        types::CompactString.encode(buf, &self.instance_id)?;
        types::CompactString.encode(buf, &self.rack_id)?;
        types::Int32.encode(buf, &self.rebalance_timeout_ms)?;
        types::CompactArray(types::CompactString).encode(buf, &self.subscribed_topic_names)?;
        types::CompactString.encode(buf, &self.subscribed_topic_regex)?;
        types::CompactString.encode(buf, &self.server_assignor)?;
        types::CompactArray(types::Struct { version }).encode(buf, &self.client_assignors)?;
        types::CompactArray(types::Struct { version }).encode(buf, &self.topic_partitions)?;
        let num_tagged_fields = self.unknown_tagged_fields.len();
        if num_tagged_fields > std::u32::MAX as usize {
            error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
            return Err(EncodeError);
        }
        types::UnsignedVarInt.encode(buf, num_tagged_fields as u32)?;

        write_unknown_tagged_fields(buf, 0.., &self.unknown_tagged_fields)?;
        Ok(())
    }
    fn compute_size(&self, version: i16) -> Result<usize, EncodeError> {
        let mut total_size = 0;
        total_size += types::CompactString.compute_size(&self.group_id)?;
        total_size += types::CompactString.compute_size(&self.member_id)?;
        total_size += types::Int32.compute_size(&self.member_epoch)?;
        total_size += types::CompactString.compute_size(&self.instance_id)?;
        total_size += types::CompactString.compute_size(&self.rack_id)?;
        total_size += types::Int32.compute_size(&self.rebalance_timeout_ms)?;
        total_size += types::CompactArray(types::CompactString).compute_size(&self.subscribed_topic_names)?;
        total_size += types::CompactString.compute_size(&self.subscribed_topic_regex)?;
        total_size += types::CompactString.compute_size(&self.server_assignor)?;
        total_size += types::CompactArray(types::Struct { version }).compute_size(&self.client_assignors)?;
        total_size += types::CompactArray(types::Struct { version }).compute_size(&self.topic_partitions)?;
        let num_tagged_fields = self.unknown_tagged_fields.len();
        if num_tagged_fields > std::u32::MAX as usize {
            error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
            return Err(EncodeError);
        }
        total_size += types::UnsignedVarInt.compute_size(num_tagged_fields as u32)?;

        total_size += compute_unknown_tagged_fields_size(&self.unknown_tagged_fields)?;
        Ok(total_size)
    }
}

impl Decodable for ConsumerGroupHeartbeatRequest {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let group_id = types::CompactString.decode(buf)?;
        let member_id = types::CompactString.decode(buf)?;
        let member_epoch = types::Int32.decode(buf)?;
        let instance_id = types::CompactString.decode(buf)?;
        let rack_id = types::CompactString.decode(buf)?;
        let rebalance_timeout_ms = types::Int32.decode(buf)?;
        let subscribed_topic_names = types::CompactArray(types::CompactString).decode(buf)?;
        let subscribed_topic_regex = types::CompactString.decode(buf)?;
        let server_assignor = types::CompactString.decode(buf)?;
        let client_assignors = types::CompactArray(types::Struct { version }).decode(buf)?;
        let topic_partitions = types::CompactArray(types::Struct { version }).decode(buf)?;
        let mut unknown_tagged_fields = BTreeMap::new();
        let num_tagged_fields = types::UnsignedVarInt.decode(buf)?;
        for _ in 0..num_tagged_fields {
            let tag: u32 = types::UnsignedVarInt.decode(buf)?;
            let size: u32 = types::UnsignedVarInt.decode(buf)?;
            let mut unknown_value = vec![0; size as usize];
            buf.try_copy_to_slice(&mut unknown_value)?;
            unknown_tagged_fields.insert(tag as i32, unknown_value);
        }
        Ok(Self {
            group_id,
            member_id,
            member_epoch,
            instance_id,
            rack_id,
            rebalance_timeout_ms,
            subscribed_topic_names,
            subscribed_topic_regex,
            server_assignor,
            client_assignors,
            topic_partitions,
            unknown_tagged_fields,
        })
    }
}

impl Default for ConsumerGroupHeartbeatRequest {
    fn default() -> Self {
        Self {
            group_id: Default::default(),
            member_id: Default::default(),
            member_epoch: 0,
            instance_id: None,
            rack_id: None,
            rebalance_timeout_ms: -1,
            subscribed_topic_names: None,
            subscribed_topic_regex: None,
            server_assignor: None,
            client_assignors: None,
            topic_partitions: None,
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for ConsumerGroupHeartbeatRequest {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 0 };
}

impl HeaderVersion for ConsumerGroupHeartbeatRequest {
    fn header_version(version: i16) -> i16 {
        2
    }
}

