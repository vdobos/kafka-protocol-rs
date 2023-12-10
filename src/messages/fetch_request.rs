//! FetchRequest
//!
//! See the schema for this message [here](https://github.com/apache/kafka/blob/trunk/clients/src/main/resources/common/message/FetchRequest.json).
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


/// Valid versions: 0-15
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct ReplicaState {
    /// The replica ID of the follower, or -1 if this request is from a consumer.
    /// 
    /// Supported API versions: 15
    pub replica_id: super::BrokerId,

    /// The epoch of this follower, or -1 if not available.
    /// 
    /// Supported API versions: 15
    pub replica_epoch: i64,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for ReplicaState {
    type Builder = ReplicaStateBuilder;

    fn builder() -> Self::Builder{
        ReplicaStateBuilder::default()
    }
}

impl Encodable for ReplicaState {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        if version >= 15 {
            types::Int32.encode(buf, &self.replica_id)?;
        } else {
            if self.replica_id != -1 {
                return Err(EncodeError)
            }
        }
        if version >= 15 {
            types::Int64.encode(buf, &self.replica_epoch)?;
        } else {
            if self.replica_epoch != -1 {
                return Err(EncodeError)
            }
        }
        if version >= 12 {
            let num_tagged_fields = self.unknown_tagged_fields.len();
            if num_tagged_fields > std::u32::MAX as usize {
                error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
                return Err(EncodeError);
            }
            types::UnsignedVarInt.encode(buf, num_tagged_fields as u32)?;

            write_unknown_tagged_fields(buf, 0.., &self.unknown_tagged_fields)?;
        }
        Ok(())
    }
    fn compute_size(&self, version: i16) -> Result<usize, EncodeError> {
        let mut total_size = 0;
        if version >= 15 {
            total_size += types::Int32.compute_size(&self.replica_id)?;
        } else {
            if self.replica_id != -1 {
                return Err(EncodeError)
            }
        }
        if version >= 15 {
            total_size += types::Int64.compute_size(&self.replica_epoch)?;
        } else {
            if self.replica_epoch != -1 {
                return Err(EncodeError)
            }
        }
        if version >= 12 {
            let num_tagged_fields = self.unknown_tagged_fields.len();
            if num_tagged_fields > std::u32::MAX as usize {
                error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
                return Err(EncodeError);
            }
            total_size += types::UnsignedVarInt.compute_size(num_tagged_fields as u32)?;

            total_size += compute_unknown_tagged_fields_size(&self.unknown_tagged_fields)?;
        }
        Ok(total_size)
    }
}

impl Decodable for ReplicaState {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let replica_id = if version >= 15 {
            types::Int32.decode(buf)?
        } else {
            (-1).into()
        };
        let replica_epoch = if version >= 15 {
            types::Int64.decode(buf)?
        } else {
            -1
        };
        let mut unknown_tagged_fields = BTreeMap::new();
        if version >= 12 {
            let num_tagged_fields = types::UnsignedVarInt.decode(buf)?;
            for _ in 0..num_tagged_fields {
                let tag: u32 = types::UnsignedVarInt.decode(buf)?;
                let size: u32 = types::UnsignedVarInt.decode(buf)?;
                let mut unknown_value = vec![0; size as usize];
                buf.try_copy_to_slice(&mut unknown_value)?;
                unknown_tagged_fields.insert(tag as i32, unknown_value);
            }
        }
        Ok(Self {
            replica_id,
            replica_epoch,
            unknown_tagged_fields,
        })
    }
}

impl Default for ReplicaState {
    fn default() -> Self {
        Self {
            replica_id: (-1).into(),
            replica_epoch: -1,
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for ReplicaState {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 15 };
}

/// Valid versions: 0-15
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct FetchPartition {
    /// The partition index.
    /// 
    /// Supported API versions: 0-15
    pub partition: i32,

    /// The current leader epoch of the partition.
    /// 
    /// Supported API versions: 9-15
    pub current_leader_epoch: i32,

    /// The message offset.
    /// 
    /// Supported API versions: 0-15
    pub fetch_offset: i64,

    /// The epoch of the last fetched record or -1 if there is none
    /// 
    /// Supported API versions: 12-15
    pub last_fetched_epoch: i32,

    /// The earliest available offset of the follower replica.  The field is only used when the request is sent by the follower.
    /// 
    /// Supported API versions: 5-15
    pub log_start_offset: i64,

    /// The maximum bytes to fetch from this partition.  See KIP-74 for cases where this limit may not be honored.
    /// 
    /// Supported API versions: 0-15
    pub partition_max_bytes: i32,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for FetchPartition {
    type Builder = FetchPartitionBuilder;

    fn builder() -> Self::Builder{
        FetchPartitionBuilder::default()
    }
}

impl Encodable for FetchPartition {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        types::Int32.encode(buf, &self.partition)?;
        if version >= 9 {
            types::Int32.encode(buf, &self.current_leader_epoch)?;
        }
        types::Int64.encode(buf, &self.fetch_offset)?;
        if version >= 12 {
            types::Int32.encode(buf, &self.last_fetched_epoch)?;
        } else {
            if self.last_fetched_epoch != -1 {
                return Err(EncodeError)
            }
        }
        if version >= 5 {
            types::Int64.encode(buf, &self.log_start_offset)?;
        }
        types::Int32.encode(buf, &self.partition_max_bytes)?;
        if version >= 12 {
            let num_tagged_fields = self.unknown_tagged_fields.len();
            if num_tagged_fields > std::u32::MAX as usize {
                error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
                return Err(EncodeError);
            }
            types::UnsignedVarInt.encode(buf, num_tagged_fields as u32)?;

            write_unknown_tagged_fields(buf, 0.., &self.unknown_tagged_fields)?;
        }
        Ok(())
    }
    fn compute_size(&self, version: i16) -> Result<usize, EncodeError> {
        let mut total_size = 0;
        total_size += types::Int32.compute_size(&self.partition)?;
        if version >= 9 {
            total_size += types::Int32.compute_size(&self.current_leader_epoch)?;
        }
        total_size += types::Int64.compute_size(&self.fetch_offset)?;
        if version >= 12 {
            total_size += types::Int32.compute_size(&self.last_fetched_epoch)?;
        } else {
            if self.last_fetched_epoch != -1 {
                return Err(EncodeError)
            }
        }
        if version >= 5 {
            total_size += types::Int64.compute_size(&self.log_start_offset)?;
        }
        total_size += types::Int32.compute_size(&self.partition_max_bytes)?;
        if version >= 12 {
            let num_tagged_fields = self.unknown_tagged_fields.len();
            if num_tagged_fields > std::u32::MAX as usize {
                error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
                return Err(EncodeError);
            }
            total_size += types::UnsignedVarInt.compute_size(num_tagged_fields as u32)?;

            total_size += compute_unknown_tagged_fields_size(&self.unknown_tagged_fields)?;
        }
        Ok(total_size)
    }
}

impl Decodable for FetchPartition {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let partition = types::Int32.decode(buf)?;
        let current_leader_epoch = if version >= 9 {
            types::Int32.decode(buf)?
        } else {
            -1
        };
        let fetch_offset = types::Int64.decode(buf)?;
        let last_fetched_epoch = if version >= 12 {
            types::Int32.decode(buf)?
        } else {
            -1
        };
        let log_start_offset = if version >= 5 {
            types::Int64.decode(buf)?
        } else {
            -1
        };
        let partition_max_bytes = types::Int32.decode(buf)?;
        let mut unknown_tagged_fields = BTreeMap::new();
        if version >= 12 {
            let num_tagged_fields = types::UnsignedVarInt.decode(buf)?;
            for _ in 0..num_tagged_fields {
                let tag: u32 = types::UnsignedVarInt.decode(buf)?;
                let size: u32 = types::UnsignedVarInt.decode(buf)?;
                let mut unknown_value = vec![0; size as usize];
                buf.try_copy_to_slice(&mut unknown_value)?;
                unknown_tagged_fields.insert(tag as i32, unknown_value);
            }
        }
        Ok(Self {
            partition,
            current_leader_epoch,
            fetch_offset,
            last_fetched_epoch,
            log_start_offset,
            partition_max_bytes,
            unknown_tagged_fields,
        })
    }
}

impl Default for FetchPartition {
    fn default() -> Self {
        Self {
            partition: 0,
            current_leader_epoch: -1,
            fetch_offset: 0,
            last_fetched_epoch: -1,
            log_start_offset: -1,
            partition_max_bytes: 0,
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for FetchPartition {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 15 };
}

/// Valid versions: 0-15
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct FetchTopic {
    /// The name of the topic to fetch.
    /// 
    /// Supported API versions: 0-12
    pub topic: super::TopicName,

    /// The unique topic ID
    /// 
    /// Supported API versions: 13-15
    pub topic_id: Uuid,

    /// The partitions to fetch.
    /// 
    /// Supported API versions: 0-15
    pub partitions: Vec<FetchPartition>,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for FetchTopic {
    type Builder = FetchTopicBuilder;

    fn builder() -> Self::Builder{
        FetchTopicBuilder::default()
    }
}

impl Encodable for FetchTopic {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        if version <= 12 {
            if version >= 12 {
                types::CompactString.encode(buf, &self.topic)?;
            } else {
                types::String.encode(buf, &self.topic)?;
            }
        }
        if version >= 13 {
            types::Uuid.encode(buf, &self.topic_id)?;
        }
        if version >= 12 {
            types::CompactArray(types::Struct { version }).encode(buf, &self.partitions)?;
        } else {
            types::Array(types::Struct { version }).encode(buf, &self.partitions)?;
        }
        if version >= 12 {
            let num_tagged_fields = self.unknown_tagged_fields.len();
            if num_tagged_fields > std::u32::MAX as usize {
                error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
                return Err(EncodeError);
            }
            types::UnsignedVarInt.encode(buf, num_tagged_fields as u32)?;

            write_unknown_tagged_fields(buf, 0.., &self.unknown_tagged_fields)?;
        }
        Ok(())
    }
    fn compute_size(&self, version: i16) -> Result<usize, EncodeError> {
        let mut total_size = 0;
        if version <= 12 {
            if version >= 12 {
                total_size += types::CompactString.compute_size(&self.topic)?;
            } else {
                total_size += types::String.compute_size(&self.topic)?;
            }
        }
        if version >= 13 {
            total_size += types::Uuid.compute_size(&self.topic_id)?;
        }
        if version >= 12 {
            total_size += types::CompactArray(types::Struct { version }).compute_size(&self.partitions)?;
        } else {
            total_size += types::Array(types::Struct { version }).compute_size(&self.partitions)?;
        }
        if version >= 12 {
            let num_tagged_fields = self.unknown_tagged_fields.len();
            if num_tagged_fields > std::u32::MAX as usize {
                error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
                return Err(EncodeError);
            }
            total_size += types::UnsignedVarInt.compute_size(num_tagged_fields as u32)?;

            total_size += compute_unknown_tagged_fields_size(&self.unknown_tagged_fields)?;
        }
        Ok(total_size)
    }
}

impl Decodable for FetchTopic {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let topic = if version <= 12 {
            if version >= 12 {
                types::CompactString.decode(buf)?
            } else {
                types::String.decode(buf)?
            }
        } else {
            Default::default()
        };
        let topic_id = if version >= 13 {
            types::Uuid.decode(buf)?
        } else {
            Uuid::nil()
        };
        let partitions = if version >= 12 {
            types::CompactArray(types::Struct { version }).decode(buf)?
        } else {
            types::Array(types::Struct { version }).decode(buf)?
        };
        let mut unknown_tagged_fields = BTreeMap::new();
        if version >= 12 {
            let num_tagged_fields = types::UnsignedVarInt.decode(buf)?;
            for _ in 0..num_tagged_fields {
                let tag: u32 = types::UnsignedVarInt.decode(buf)?;
                let size: u32 = types::UnsignedVarInt.decode(buf)?;
                let mut unknown_value = vec![0; size as usize];
                buf.try_copy_to_slice(&mut unknown_value)?;
                unknown_tagged_fields.insert(tag as i32, unknown_value);
            }
        }
        Ok(Self {
            topic,
            topic_id,
            partitions,
            unknown_tagged_fields,
        })
    }
}

impl Default for FetchTopic {
    fn default() -> Self {
        Self {
            topic: Default::default(),
            topic_id: Uuid::nil(),
            partitions: Default::default(),
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for FetchTopic {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 15 };
}

/// Valid versions: 0-15
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct ForgottenTopic {
    /// The topic name.
    /// 
    /// Supported API versions: 7-12
    pub topic: super::TopicName,

    /// The unique topic ID
    /// 
    /// Supported API versions: 13-15
    pub topic_id: Uuid,

    /// The partitions indexes to forget.
    /// 
    /// Supported API versions: 7-15
    pub partitions: Vec<i32>,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for ForgottenTopic {
    type Builder = ForgottenTopicBuilder;

    fn builder() -> Self::Builder{
        ForgottenTopicBuilder::default()
    }
}

impl Encodable for ForgottenTopic {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        if version >= 7 && version <= 12 {
            if version >= 12 {
                types::CompactString.encode(buf, &self.topic)?;
            } else {
                types::String.encode(buf, &self.topic)?;
            }
        }
        if version >= 13 {
            types::Uuid.encode(buf, &self.topic_id)?;
        }
        if version >= 7 {
            if version >= 12 {
                types::CompactArray(types::Int32).encode(buf, &self.partitions)?;
            } else {
                types::Array(types::Int32).encode(buf, &self.partitions)?;
            }
        } else {
            if !self.partitions.is_empty() {
                return Err(EncodeError)
            }
        }
        if version >= 12 {
            let num_tagged_fields = self.unknown_tagged_fields.len();
            if num_tagged_fields > std::u32::MAX as usize {
                error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
                return Err(EncodeError);
            }
            types::UnsignedVarInt.encode(buf, num_tagged_fields as u32)?;

            write_unknown_tagged_fields(buf, 0.., &self.unknown_tagged_fields)?;
        }
        Ok(())
    }
    fn compute_size(&self, version: i16) -> Result<usize, EncodeError> {
        let mut total_size = 0;
        if version >= 7 && version <= 12 {
            if version >= 12 {
                total_size += types::CompactString.compute_size(&self.topic)?;
            } else {
                total_size += types::String.compute_size(&self.topic)?;
            }
        }
        if version >= 13 {
            total_size += types::Uuid.compute_size(&self.topic_id)?;
        }
        if version >= 7 {
            if version >= 12 {
                total_size += types::CompactArray(types::Int32).compute_size(&self.partitions)?;
            } else {
                total_size += types::Array(types::Int32).compute_size(&self.partitions)?;
            }
        } else {
            if !self.partitions.is_empty() {
                return Err(EncodeError)
            }
        }
        if version >= 12 {
            let num_tagged_fields = self.unknown_tagged_fields.len();
            if num_tagged_fields > std::u32::MAX as usize {
                error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
                return Err(EncodeError);
            }
            total_size += types::UnsignedVarInt.compute_size(num_tagged_fields as u32)?;

            total_size += compute_unknown_tagged_fields_size(&self.unknown_tagged_fields)?;
        }
        Ok(total_size)
    }
}

impl Decodable for ForgottenTopic {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let topic = if version >= 7 && version <= 12 {
            if version >= 12 {
                types::CompactString.decode(buf)?
            } else {
                types::String.decode(buf)?
            }
        } else {
            Default::default()
        };
        let topic_id = if version >= 13 {
            types::Uuid.decode(buf)?
        } else {
            Uuid::nil()
        };
        let partitions = if version >= 7 {
            if version >= 12 {
                types::CompactArray(types::Int32).decode(buf)?
            } else {
                types::Array(types::Int32).decode(buf)?
            }
        } else {
            Default::default()
        };
        let mut unknown_tagged_fields = BTreeMap::new();
        if version >= 12 {
            let num_tagged_fields = types::UnsignedVarInt.decode(buf)?;
            for _ in 0..num_tagged_fields {
                let tag: u32 = types::UnsignedVarInt.decode(buf)?;
                let size: u32 = types::UnsignedVarInt.decode(buf)?;
                let mut unknown_value = vec![0; size as usize];
                buf.try_copy_to_slice(&mut unknown_value)?;
                unknown_tagged_fields.insert(tag as i32, unknown_value);
            }
        }
        Ok(Self {
            topic,
            topic_id,
            partitions,
            unknown_tagged_fields,
        })
    }
}

impl Default for ForgottenTopic {
    fn default() -> Self {
        Self {
            topic: Default::default(),
            topic_id: Uuid::nil(),
            partitions: Default::default(),
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for ForgottenTopic {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 15 };
}

/// Valid versions: 0-15
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct FetchRequest {
    /// The clusterId if known. This is used to validate metadata fetches prior to broker registration.
    /// 
    /// Supported API versions: 12-15
    pub cluster_id: Option<StrBytes>,

    /// The broker ID of the follower, of -1 if this request is from a consumer.
    /// 
    /// Supported API versions: 0-14
    pub replica_id: super::BrokerId,

    /// 
    /// 
    /// Supported API versions: 15
    pub replica_state: ReplicaState,

    /// The maximum time in milliseconds to wait for the response.
    /// 
    /// Supported API versions: 0-15
    pub max_wait_ms: i32,

    /// The minimum bytes to accumulate in the response.
    /// 
    /// Supported API versions: 0-15
    pub min_bytes: i32,

    /// The maximum bytes to fetch.  See KIP-74 for cases where this limit may not be honored.
    /// 
    /// Supported API versions: 3-15
    pub max_bytes: i32,

    /// This setting controls the visibility of transactional records. Using READ_UNCOMMITTED (isolation_level = 0) makes all records visible. With READ_COMMITTED (isolation_level = 1), non-transactional and COMMITTED transactional records are visible. To be more concrete, READ_COMMITTED returns all data from offsets smaller than the current LSO (last stable offset), and enables the inclusion of the list of aborted transactions in the result, which allows consumers to discard ABORTED transactional records
    /// 
    /// Supported API versions: 4-15
    pub isolation_level: i8,

    /// The fetch session ID.
    /// 
    /// Supported API versions: 7-15
    pub session_id: i32,

    /// The fetch session epoch, which is used for ordering requests in a session.
    /// 
    /// Supported API versions: 7-15
    pub session_epoch: i32,

    /// The topics to fetch.
    /// 
    /// Supported API versions: 0-15
    pub topics: Vec<FetchTopic>,

    /// In an incremental fetch request, the partitions to remove.
    /// 
    /// Supported API versions: 7-15
    pub forgotten_topics_data: Vec<ForgottenTopic>,

    /// Rack ID of the consumer making this request
    /// 
    /// Supported API versions: 11-15
    pub rack_id: StrBytes,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for FetchRequest {
    type Builder = FetchRequestBuilder;

    fn builder() -> Self::Builder{
        FetchRequestBuilder::default()
    }
}

impl Encodable for FetchRequest {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        if version <= 14 {
            types::Int32.encode(buf, &self.replica_id)?;
        } else {
            if self.replica_id != -1 {
                return Err(EncodeError)
            }
        }
        types::Int32.encode(buf, &self.max_wait_ms)?;
        types::Int32.encode(buf, &self.min_bytes)?;
        if version >= 3 {
            types::Int32.encode(buf, &self.max_bytes)?;
        }
        if version >= 4 {
            types::Int8.encode(buf, &self.isolation_level)?;
        }
        if version >= 7 {
            types::Int32.encode(buf, &self.session_id)?;
        }
        if version >= 7 {
            types::Int32.encode(buf, &self.session_epoch)?;
        }
        if version >= 12 {
            types::CompactArray(types::Struct { version }).encode(buf, &self.topics)?;
        } else {
            types::Array(types::Struct { version }).encode(buf, &self.topics)?;
        }
        if version >= 7 {
            if version >= 12 {
                types::CompactArray(types::Struct { version }).encode(buf, &self.forgotten_topics_data)?;
            } else {
                types::Array(types::Struct { version }).encode(buf, &self.forgotten_topics_data)?;
            }
        } else {
            if !self.forgotten_topics_data.is_empty() {
                return Err(EncodeError)
            }
        }
        if version >= 11 {
            if version >= 12 {
                types::CompactString.encode(buf, &self.rack_id)?;
            } else {
                types::String.encode(buf, &self.rack_id)?;
            }
        }
        if version >= 12 {
            let mut num_tagged_fields = self.unknown_tagged_fields.len();
            if !self.cluster_id.is_none() {
                num_tagged_fields += 1;
            }
            if version >= 15 {
                if &self.replica_state != &Default::default() {
                    num_tagged_fields += 1;
                }
            }if num_tagged_fields > std::u32::MAX as usize {
                error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
                return Err(EncodeError);
            }
            types::UnsignedVarInt.encode(buf, num_tagged_fields as u32)?;
            if !self.cluster_id.is_none() {
                let computed_size = types::CompactString.compute_size(&self.cluster_id)?;
                if computed_size > std::u32::MAX as usize {
                    error!("Tagged field is too large to encode ({} bytes)", computed_size);
                    return Err(EncodeError);
                }
                types::UnsignedVarInt.encode(buf, 0)?;
                types::UnsignedVarInt.encode(buf, computed_size as u32)?;
                types::CompactString.encode(buf, &self.cluster_id)?;
            }
            if version >= 15 {
                if &self.replica_state != &Default::default() {
                    let computed_size = types::Struct { version }.compute_size(&self.replica_state)?;
                    if computed_size > std::u32::MAX as usize {
                        error!("Tagged field is too large to encode ({} bytes)", computed_size);
                        return Err(EncodeError);
                    }
                    types::UnsignedVarInt.encode(buf, 1)?;
                    types::UnsignedVarInt.encode(buf, computed_size as u32)?;
                    types::Struct { version }.encode(buf, &self.replica_state)?;
                }
            }
            write_unknown_tagged_fields(buf, 2.., &self.unknown_tagged_fields)?;
        }
        Ok(())
    }
    fn compute_size(&self, version: i16) -> Result<usize, EncodeError> {
        let mut total_size = 0;
        if version <= 14 {
            total_size += types::Int32.compute_size(&self.replica_id)?;
        } else {
            if self.replica_id != -1 {
                return Err(EncodeError)
            }
        }
        total_size += types::Int32.compute_size(&self.max_wait_ms)?;
        total_size += types::Int32.compute_size(&self.min_bytes)?;
        if version >= 3 {
            total_size += types::Int32.compute_size(&self.max_bytes)?;
        }
        if version >= 4 {
            total_size += types::Int8.compute_size(&self.isolation_level)?;
        }
        if version >= 7 {
            total_size += types::Int32.compute_size(&self.session_id)?;
        }
        if version >= 7 {
            total_size += types::Int32.compute_size(&self.session_epoch)?;
        }
        if version >= 12 {
            total_size += types::CompactArray(types::Struct { version }).compute_size(&self.topics)?;
        } else {
            total_size += types::Array(types::Struct { version }).compute_size(&self.topics)?;
        }
        if version >= 7 {
            if version >= 12 {
                total_size += types::CompactArray(types::Struct { version }).compute_size(&self.forgotten_topics_data)?;
            } else {
                total_size += types::Array(types::Struct { version }).compute_size(&self.forgotten_topics_data)?;
            }
        } else {
            if !self.forgotten_topics_data.is_empty() {
                return Err(EncodeError)
            }
        }
        if version >= 11 {
            if version >= 12 {
                total_size += types::CompactString.compute_size(&self.rack_id)?;
            } else {
                total_size += types::String.compute_size(&self.rack_id)?;
            }
        }
        if version >= 12 {
            let mut num_tagged_fields = self.unknown_tagged_fields.len();
            if !self.cluster_id.is_none() {
                num_tagged_fields += 1;
            }
            if version >= 15 {
                if &self.replica_state != &Default::default() {
                    num_tagged_fields += 1;
                }
            }if num_tagged_fields > std::u32::MAX as usize {
                error!("Too many tagged fields to encode ({} fields)", num_tagged_fields);
                return Err(EncodeError);
            }
            total_size += types::UnsignedVarInt.compute_size(num_tagged_fields as u32)?;
            if !self.cluster_id.is_none() {
                let computed_size = types::CompactString.compute_size(&self.cluster_id)?;
                if computed_size > std::u32::MAX as usize {
                    error!("Tagged field is too large to encode ({} bytes)", computed_size);
                    return Err(EncodeError);
                }
                total_size += types::UnsignedVarInt.compute_size(0)?;
                total_size += types::UnsignedVarInt.compute_size(computed_size as u32)?;
                total_size += computed_size;
            }
            if version >= 15 {
                if &self.replica_state != &Default::default() {
                    let computed_size = types::Struct { version }.compute_size(&self.replica_state)?;
                    if computed_size > std::u32::MAX as usize {
                        error!("Tagged field is too large to encode ({} bytes)", computed_size);
                        return Err(EncodeError);
                    }
                    total_size += types::UnsignedVarInt.compute_size(1)?;
                    total_size += types::UnsignedVarInt.compute_size(computed_size as u32)?;
                    total_size += computed_size;
                }
            }
            total_size += compute_unknown_tagged_fields_size(&self.unknown_tagged_fields)?;
        }
        Ok(total_size)
    }
}

impl Decodable for FetchRequest {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let mut cluster_id = None;
        let replica_id = if version <= 14 {
            types::Int32.decode(buf)?
        } else {
            (-1).into()
        };
        let mut replica_state = Default::default();
        let max_wait_ms = types::Int32.decode(buf)?;
        let min_bytes = types::Int32.decode(buf)?;
        let max_bytes = if version >= 3 {
            types::Int32.decode(buf)?
        } else {
            0x7fffffff
        };
        let isolation_level = if version >= 4 {
            types::Int8.decode(buf)?
        } else {
            0
        };
        let session_id = if version >= 7 {
            types::Int32.decode(buf)?
        } else {
            0
        };
        let session_epoch = if version >= 7 {
            types::Int32.decode(buf)?
        } else {
            -1
        };
        let topics = if version >= 12 {
            types::CompactArray(types::Struct { version }).decode(buf)?
        } else {
            types::Array(types::Struct { version }).decode(buf)?
        };
        let forgotten_topics_data = if version >= 7 {
            if version >= 12 {
                types::CompactArray(types::Struct { version }).decode(buf)?
            } else {
                types::Array(types::Struct { version }).decode(buf)?
            }
        } else {
            Default::default()
        };
        let rack_id = if version >= 11 {
            if version >= 12 {
                types::CompactString.decode(buf)?
            } else {
                types::String.decode(buf)?
            }
        } else {
            StrBytes::from_str("")
        };
        let mut unknown_tagged_fields = BTreeMap::new();
        if version >= 12 {
            let num_tagged_fields = types::UnsignedVarInt.decode(buf)?;
            for _ in 0..num_tagged_fields {
                let tag: u32 = types::UnsignedVarInt.decode(buf)?;
                let size: u32 = types::UnsignedVarInt.decode(buf)?;
                match tag {
                    0 => {
                        cluster_id = types::CompactString.decode(buf)?;
                    },
                    1 => {
                        if version >= 15 {
                            replica_state = types::Struct { version }.decode(buf)?;
                        } else {
                            error!("Tag {} is not valid for version {}", tag, version);
                            return Err(DecodeError);
                        }
                    },
                    _ => {
                        let mut unknown_value = vec![0; size as usize];
                        buf.try_copy_to_slice(&mut unknown_value)?;
                        unknown_tagged_fields.insert(tag as i32, unknown_value);
                    }
                }
            }
        }
        Ok(Self {
            cluster_id,
            replica_id,
            replica_state,
            max_wait_ms,
            min_bytes,
            max_bytes,
            isolation_level,
            session_id,
            session_epoch,
            topics,
            forgotten_topics_data,
            rack_id,
            unknown_tagged_fields,
        })
    }
}

impl Default for FetchRequest {
    fn default() -> Self {
        Self {
            cluster_id: None,
            replica_id: (-1).into(),
            replica_state: Default::default(),
            max_wait_ms: 0,
            min_bytes: 0,
            max_bytes: 0x7fffffff,
            isolation_level: 0,
            session_id: 0,
            session_epoch: -1,
            topics: Default::default(),
            forgotten_topics_data: Default::default(),
            rack_id: StrBytes::from_str(""),
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for FetchRequest {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 15 };
}

impl HeaderVersion for FetchRequest {
    fn header_version(version: i16) -> i16 {
        if version >= 12 {
            2
        } else {
            1
        }
    }
}

