//! AlterPartitionRequest
//!
//! See the schema for this message [here](https://github.com/apache/kafka/blob/trunk/clients/src/main/resources/common/message/AlterPartitionRequest.json).
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


/// Valid versions: 0-3
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct BrokerState {
    /// The ID of the broker.
    /// 
    /// Supported API versions: 3
    pub broker_id: super::BrokerId,

    /// The epoch of the broker. It will be -1 if the epoch check is not supported.
    /// 
    /// Supported API versions: 3
    pub broker_epoch: i64,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for BrokerState {
    type Builder = BrokerStateBuilder;

    fn builder() -> Self::Builder{
        BrokerStateBuilder::default()
    }
}

impl Encodable for BrokerState {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        if version >= 3 {
            types::Int32.encode(buf, &self.broker_id)?;
        } else {
            if self.broker_id != 0 {
                return Err(EncodeError)
            }
        }
        if version >= 3 {
            types::Int64.encode(buf, &self.broker_epoch)?;
        } else {
            if self.broker_epoch != -1 {
                return Err(EncodeError)
            }
        }
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
        if version >= 3 {
            total_size += types::Int32.compute_size(&self.broker_id)?;
        } else {
            if self.broker_id != 0 {
                return Err(EncodeError)
            }
        }
        if version >= 3 {
            total_size += types::Int64.compute_size(&self.broker_epoch)?;
        } else {
            if self.broker_epoch != -1 {
                return Err(EncodeError)
            }
        }
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

impl Decodable for BrokerState {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let broker_id = if version >= 3 {
            types::Int32.decode(buf)?
        } else {
            (0).into()
        };
        let broker_epoch = if version >= 3 {
            types::Int64.decode(buf)?
        } else {
            -1
        };
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
            broker_id,
            broker_epoch,
            unknown_tagged_fields,
        })
    }
}

impl Default for BrokerState {
    fn default() -> Self {
        Self {
            broker_id: (0).into(),
            broker_epoch: -1,
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for BrokerState {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 3 };
}

/// Valid versions: 0-3
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct PartitionData {
    /// The partition index
    /// 
    /// Supported API versions: 0-3
    pub partition_index: i32,

    /// The leader epoch of this partition
    /// 
    /// Supported API versions: 0-3
    pub leader_epoch: i32,

    /// The ISR for this partition. Deprecated since version 3.
    /// 
    /// Supported API versions: 0-2
    pub new_isr: Vec<super::BrokerId>,

    /// 
    /// 
    /// Supported API versions: 3
    pub new_isr_with_epochs: Vec<BrokerState>,

    /// 1 if the partition is recovering from an unclean leader election; 0 otherwise.
    /// 
    /// Supported API versions: 1-3
    pub leader_recovery_state: i8,

    /// The expected epoch of the partition which is being updated. For legacy cluster this is the ZkVersion in the LeaderAndIsr request.
    /// 
    /// Supported API versions: 0-3
    pub partition_epoch: i32,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for PartitionData {
    type Builder = PartitionDataBuilder;

    fn builder() -> Self::Builder{
        PartitionDataBuilder::default()
    }
}

impl Encodable for PartitionData {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        types::Int32.encode(buf, &self.partition_index)?;
        types::Int32.encode(buf, &self.leader_epoch)?;
        if version <= 2 {
            types::CompactArray(types::Int32).encode(buf, &self.new_isr)?;
        } else {
            if !self.new_isr.is_empty() {
                return Err(EncodeError)
            }
        }
        if version >= 3 {
            types::CompactArray(types::Struct { version }).encode(buf, &self.new_isr_with_epochs)?;
        } else {
            if !self.new_isr_with_epochs.is_empty() {
                return Err(EncodeError)
            }
        }
        if version >= 1 {
            types::Int8.encode(buf, &self.leader_recovery_state)?;
        } else {
            if self.leader_recovery_state != 0 {
                return Err(EncodeError)
            }
        }
        types::Int32.encode(buf, &self.partition_epoch)?;
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
        total_size += types::Int32.compute_size(&self.partition_index)?;
        total_size += types::Int32.compute_size(&self.leader_epoch)?;
        if version <= 2 {
            total_size += types::CompactArray(types::Int32).compute_size(&self.new_isr)?;
        } else {
            if !self.new_isr.is_empty() {
                return Err(EncodeError)
            }
        }
        if version >= 3 {
            total_size += types::CompactArray(types::Struct { version }).compute_size(&self.new_isr_with_epochs)?;
        } else {
            if !self.new_isr_with_epochs.is_empty() {
                return Err(EncodeError)
            }
        }
        if version >= 1 {
            total_size += types::Int8.compute_size(&self.leader_recovery_state)?;
        } else {
            if self.leader_recovery_state != 0 {
                return Err(EncodeError)
            }
        }
        total_size += types::Int32.compute_size(&self.partition_epoch)?;
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

impl Decodable for PartitionData {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let partition_index = types::Int32.decode(buf)?;
        let leader_epoch = types::Int32.decode(buf)?;
        let new_isr = if version <= 2 {
            types::CompactArray(types::Int32).decode(buf)?
        } else {
            Default::default()
        };
        let new_isr_with_epochs = if version >= 3 {
            types::CompactArray(types::Struct { version }).decode(buf)?
        } else {
            Default::default()
        };
        let leader_recovery_state = if version >= 1 {
            types::Int8.decode(buf)?
        } else {
            0
        };
        let partition_epoch = types::Int32.decode(buf)?;
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
            partition_index,
            leader_epoch,
            new_isr,
            new_isr_with_epochs,
            leader_recovery_state,
            partition_epoch,
            unknown_tagged_fields,
        })
    }
}

impl Default for PartitionData {
    fn default() -> Self {
        Self {
            partition_index: 0,
            leader_epoch: 0,
            new_isr: Default::default(),
            new_isr_with_epochs: Default::default(),
            leader_recovery_state: 0,
            partition_epoch: 0,
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for PartitionData {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 3 };
}

/// Valid versions: 0-3
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct TopicData {
    /// The name of the topic to alter ISRs for
    /// 
    /// Supported API versions: 0-1
    pub topic_name: super::TopicName,

    /// The ID of the topic to alter ISRs for
    /// 
    /// Supported API versions: 2-3
    pub topic_id: Uuid,

    /// 
    /// 
    /// Supported API versions: 0-3
    pub partitions: Vec<PartitionData>,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for TopicData {
    type Builder = TopicDataBuilder;

    fn builder() -> Self::Builder{
        TopicDataBuilder::default()
    }
}

impl Encodable for TopicData {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        if version <= 1 {
            types::CompactString.encode(buf, &self.topic_name)?;
        }
        if version >= 2 {
            types::Uuid.encode(buf, &self.topic_id)?;
        }
        types::CompactArray(types::Struct { version }).encode(buf, &self.partitions)?;
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
        if version <= 1 {
            total_size += types::CompactString.compute_size(&self.topic_name)?;
        }
        if version >= 2 {
            total_size += types::Uuid.compute_size(&self.topic_id)?;
        }
        total_size += types::CompactArray(types::Struct { version }).compute_size(&self.partitions)?;
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

impl Decodable for TopicData {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let topic_name = if version <= 1 {
            types::CompactString.decode(buf)?
        } else {
            Default::default()
        };
        let topic_id = if version >= 2 {
            types::Uuid.decode(buf)?
        } else {
            Uuid::nil()
        };
        let partitions = types::CompactArray(types::Struct { version }).decode(buf)?;
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
            topic_name,
            topic_id,
            partitions,
            unknown_tagged_fields,
        })
    }
}

impl Default for TopicData {
    fn default() -> Self {
        Self {
            topic_name: Default::default(),
            topic_id: Uuid::nil(),
            partitions: Default::default(),
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for TopicData {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 3 };
}

/// Valid versions: 0-3
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, derive_builder::Builder)]
#[builder(default)]
pub struct AlterPartitionRequest {
    /// The ID of the requesting broker
    /// 
    /// Supported API versions: 0-3
    pub broker_id: super::BrokerId,

    /// The epoch of the requesting broker
    /// 
    /// Supported API versions: 0-3
    pub broker_epoch: i64,

    /// 
    /// 
    /// Supported API versions: 0-3
    pub topics: Vec<TopicData>,

    /// Other tagged fields
    pub unknown_tagged_fields: BTreeMap<i32, Vec<u8>>,
}

impl Builder for AlterPartitionRequest {
    type Builder = AlterPartitionRequestBuilder;

    fn builder() -> Self::Builder{
        AlterPartitionRequestBuilder::default()
    }
}

impl Encodable for AlterPartitionRequest {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, version: i16) -> Result<(), EncodeError> {
        types::Int32.encode(buf, &self.broker_id)?;
        types::Int64.encode(buf, &self.broker_epoch)?;
        types::CompactArray(types::Struct { version }).encode(buf, &self.topics)?;
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
        total_size += types::Int32.compute_size(&self.broker_id)?;
        total_size += types::Int64.compute_size(&self.broker_epoch)?;
        total_size += types::CompactArray(types::Struct { version }).compute_size(&self.topics)?;
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

impl Decodable for AlterPartitionRequest {
    fn decode<B: ByteBuf>(buf: &mut B, version: i16) -> Result<Self, DecodeError> {
        let broker_id = types::Int32.decode(buf)?;
        let broker_epoch = types::Int64.decode(buf)?;
        let topics = types::CompactArray(types::Struct { version }).decode(buf)?;
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
            broker_id,
            broker_epoch,
            topics,
            unknown_tagged_fields,
        })
    }
}

impl Default for AlterPartitionRequest {
    fn default() -> Self {
        Self {
            broker_id: (0).into(),
            broker_epoch: -1,
            topics: Default::default(),
            unknown_tagged_fields: BTreeMap::new(),
        }
    }
}

impl Message for AlterPartitionRequest {
    const VERSIONS: VersionRange = VersionRange { min: 0, max: 3 };
}

impl HeaderVersion for AlterPartitionRequest {
    fn header_version(version: i16) -> i16 {
        2
    }
}

