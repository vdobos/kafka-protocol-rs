//! Raw types of the Kafka protocol, as defined by the protocol specification.
//!
//! In general, most types map closely to a corresponding rust type, with the exception of a number
//! types that use zigzag encoded to represent length as a "compact" representation.
//!
//! It is unnecessary to interact directly with these types for most use cases.
use std::hash::Hash;
use std::string::String as StdString;

use indexmap::IndexMap;
use string::TryFrom;

use super::{
    Decodable, DecodeError, Decoder, Encodable, EncodeError, Encoder, MapDecodable, MapEncodable,
    NewType, StrBytes,
};
use crate::protocol::buf::{ByteBuf, ByteBufMut};

macro_rules! define_copy_impl {
    ($e:ident, $t:ty) => (
        impl Encoder<$t> for $e {
            fn encode<B: ByteBufMut>(&self, buf: &mut B, value: $t) -> Result<(), EncodeError> {
                self.encode(buf, &value)
            }
            fn compute_size(&self, value: $t) -> Result<usize, EncodeError> {
                self.compute_size(&value)
            }
            fn fixed_size(&self) -> Option<usize> {
                <Self as Encoder<&$t>>::fixed_size(self)
            }
        }
    )
}

/// A boolean value.
#[derive(Debug, Copy, Clone, Default)]
pub struct Boolean;

impl<T: NewType<bool>> Encoder<&T> for Boolean {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &T) -> Result<(), EncodeError> {
        buf.put_u8(if *value.borrow() { 1 } else { 0 });
        Ok(())
    }
    fn compute_size(&self, _value: &T) -> Result<usize, EncodeError> {
        Ok(1)
    }
    fn fixed_size(&self) -> Option<usize> {
        Some(1)
    }
}

impl<T: NewType<bool>> Decoder<T> for Boolean {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<T, DecodeError> {
        Ok((buf.try_get_u8()? != 0).into())
    }
}

define_copy_impl!(Boolean, bool);

macro_rules! define_simple_ints {
    ($($name:ident: $t:ty [$put:ident, $get:ident],)*) => (
        $(
            /// A struct representing [`$ty`]
            #[derive(Debug, Copy, Clone)]
            pub struct $name;

            impl<T: NewType<$t>> Encoder<&T> for $name {
                fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &T) -> Result<(), EncodeError> {
                    Ok(buf.$put(*value.borrow()))
                }
                fn compute_size(&self, _value: &T) -> Result<usize, EncodeError> {
                    Ok(std::mem::size_of::<$t>())
                }
                fn fixed_size(&self) -> Option<usize> {
                    Some(std::mem::size_of::<$t>())
                }
            }

            impl<T: NewType<$t>> Decoder<T> for $name {
                fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<T, DecodeError> {
                    Ok(buf.$get()?.into())
                }
            }

            define_copy_impl!($name, $t);
        )*
    )
}

define_simple_ints! {
    Int8: i8 [put_i8, try_get_i8],
    Int16: i16 [put_i16, try_get_i16],
    UInt16: u16 [put_u16, try_get_u16],
    Int32: i32 [put_i32, try_get_i32],
    Int64: i64 [put_i64, try_get_i64],
    UInt32: u32 [put_u32, try_get_u32],
    Float64: f64 [put_f64, try_get_f64],
}

/// An unsigned zigzag encoded int.
#[derive(Debug, Copy, Clone, Default)]
pub struct UnsignedVarInt;

impl<T: NewType<u32>> Encoder<&T> for UnsignedVarInt {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &T) -> Result<(), EncodeError> {
        let mut value = *value.borrow();
        while value >= 0x80 {
            buf.put_u8((value as u8) | 0x80);
            value >>= 7;
        }
        buf.put_u8(value as u8);
        Ok(())
    }
    fn compute_size(&self, value: &T) -> Result<usize, EncodeError> {
        let value = *value.borrow();
        Ok(match value {
            0x0..=0x7f => 1,
            0x80..=0x3fff => 2,
            0x4000..=0x1fffff => 3,
            0x200000..=0xfffffff => 4,
            0x10000000..=0xffffffff => 5,
        })
    }
}

impl<T: NewType<u32>> Decoder<T> for UnsignedVarInt {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<T, DecodeError> {
        let mut value = 0;
        for i in 0..5 {
            let b = buf.try_get_u8()? as u32;
            value |= (b & 0x7F) << (i * 7);
            if b < 0x80 {
                break;
            }
        }
        Ok(value.into())
    }
}

define_copy_impl!(UnsignedVarInt, u32);

/// An unsigned zigzag encoded long.
#[derive(Debug, Copy, Clone, Default)]
pub struct UnsignedVarLong;

impl<T: NewType<u64>> Encoder<&T> for UnsignedVarLong {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &T) -> Result<(), EncodeError> {
        let mut value = *value.borrow();
        while value >= 0x80 {
            buf.put_u8((value as u8) | 0x80);
            value >>= 7;
        }
        buf.put_u8(value as u8);
        Ok(())
    }
    fn compute_size(&self, value: &T) -> Result<usize, EncodeError> {
        let value = *value.borrow();
        Ok(match value {
            0x0..=0x7f => 1,
            0x80..=0x3fff => 2,
            0x4000..=0x1fffff => 3,
            0x200000..=0xfffffff => 4,
            0x10000000..=0x7ffffffff => 5,
            0x800000000..=0x3ffffffffff => 6,
            0x40000000000..=0x1ffffffffffff => 7,
            0x2000000000000..=0xffffffffffffff => 8,
            0x100000000000000..=0x7fffffffffffffff => 9,
            0x8000000000000000..=0xffffffffffffffff => 10,
        })
    }
}

impl<T: NewType<u64>> Decoder<T> for UnsignedVarLong {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<T, DecodeError> {
        let mut value = 0;
        for i in 0..10 {
            let b = buf.try_get_u8()? as u64;
            value |= (b & 0x7F) << (i * 7);
            if b < 0x80 {
                break;
            }
        }
        Ok(value.into())
    }
}

define_copy_impl!(UnsignedVarLong, u64);

/// A zizag encoded int.
#[derive(Debug, Copy, Clone, Default)]
pub struct VarInt;

impl<T: NewType<i32>> Encoder<&T> for VarInt {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &T) -> Result<(), EncodeError> {
        let value = *value.borrow();
        let zigzag = ((value << 1) ^ (value >> 31)) as u32;
        UnsignedVarInt.encode(buf, zigzag)
    }
    fn compute_size(&self, value: &T) -> Result<usize, EncodeError> {
        let value = *value.borrow();
        let zigzag = ((value << 1) ^ (value >> 31)) as u32;
        UnsignedVarInt.compute_size(zigzag)
    }
}

impl<T: NewType<i32>> Decoder<T> for VarInt {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<T, DecodeError> {
        let zigzag: u32 = UnsignedVarInt.decode(buf)?;
        Ok((((zigzag >> 1) as i32) ^ (-((zigzag & 1) as i32))).into())
    }
}

define_copy_impl!(VarInt, i32);

/// A zizag encoded long.
#[derive(Debug, Copy, Clone, Default)]
pub struct VarLong;

impl<T: NewType<i64>> Encoder<&T> for VarLong {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &T) -> Result<(), EncodeError> {
        let value = *value.borrow();
        let zigzag = ((value << 1) ^ (value >> 63)) as u64;
        UnsignedVarLong.encode(buf, &zigzag)
    }
    fn compute_size(&self, value: &T) -> Result<usize, EncodeError> {
        let value = *value.borrow();
        let zigzag = ((value << 1) ^ (value >> 63)) as u64;
        UnsignedVarLong.compute_size(zigzag)
    }
}

impl<T: NewType<i64>> Decoder<T> for VarLong {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<T, DecodeError> {
        let zigzag: u64 = UnsignedVarLong.decode(buf)?;
        Ok((((zigzag >> 1) as i64) ^ (-((zigzag & 1) as i64))).into())
    }
}

define_copy_impl!(VarLong, i64);

/// A v4 UUID.
#[derive(Debug, Copy, Clone, Default)]
pub struct Uuid;

impl<T: NewType<uuid::Uuid>> Encoder<&T> for Uuid {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &T) -> Result<(), EncodeError> {
        buf.put_slice(value.borrow().as_bytes());
        Ok(())
    }
    fn compute_size(&self, _value: &T) -> Result<usize, EncodeError> {
        Ok(16)
    }
    fn fixed_size(&self) -> Option<usize> {
        Some(16)
    }
}

impl<T: NewType<uuid::Uuid>> Decoder<T> for Uuid {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<T, DecodeError> {
        let mut result = [0; 16];
        buf.try_copy_to_slice(&mut result)?;
        Ok(uuid::Uuid::from_bytes(result).into())
    }
}

define_copy_impl!(Uuid, uuid::Uuid);

/// A string of length up to [`i16::MAX`].
#[derive(Debug, Copy, Clone, Default)]
pub struct String;

impl Encoder<Option<&str>> for String {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: Option<&str>) -> Result<(), EncodeError> {
        if let Some(s) = value {
            if s.len() > std::i16::MAX as usize {
                error!("String is too long to encode ({} bytes)", s.len());
                Err(EncodeError)
            } else {
                Int16.encode(buf, s.len() as i16)?;
                buf.put_slice(s.as_bytes());
                Ok(())
            }
        } else {
            Int16.encode(buf, -1)?;
            Ok(())
        }
    }
    fn compute_size(&self, value: Option<&str>) -> Result<usize, EncodeError> {
        if let Some(s) = value {
            if s.len() > std::i16::MAX as usize {
                error!("String is too long to encode ({} bytes)", s.len());
                Err(EncodeError)
            } else {
                Ok(2 + s.len())
            }
        } else {
            Ok(2)
        }
    }
}

impl Encoder<Option<&StdString>> for String {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: Option<&StdString>,
    ) -> Result<(), EncodeError> {
        String.encode(buf, value.map(|s| s.as_str()))
    }
    fn compute_size(&self, value: Option<&StdString>) -> Result<usize, EncodeError> {
        String.compute_size(value.map(|s| s.as_str()))
    }
}

impl Encoder<&Option<StdString>> for String {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &Option<StdString>,
    ) -> Result<(), EncodeError> {
        String.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<StdString>) -> Result<usize, EncodeError> {
        String.compute_size(value.as_ref())
    }
}

impl<T: NewType<StrBytes>> Encoder<Option<&T>> for String {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: Option<&T>) -> Result<(), EncodeError> {
        String.encode(buf, value.map(|s| &**s.borrow()))
    }
    fn compute_size(&self, value: Option<&T>) -> Result<usize, EncodeError> {
        String.compute_size(value.map(|s| &**s.borrow()))
    }
}

impl<T: NewType<StrBytes>> Encoder<&Option<T>> for String {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &Option<T>) -> Result<(), EncodeError> {
        String.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<T>) -> Result<usize, EncodeError> {
        String.compute_size(value.as_ref())
    }
}

impl Encoder<&str> for String {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &str) -> Result<(), EncodeError> {
        String.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &str) -> Result<usize, EncodeError> {
        String.compute_size(Some(value))
    }
}

impl Encoder<&StdString> for String {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &StdString) -> Result<(), EncodeError> {
        String.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &StdString) -> Result<usize, EncodeError> {
        String.compute_size(Some(value))
    }
}

impl<T: NewType<StrBytes>> Encoder<&T> for String {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &T) -> Result<(), EncodeError> {
        String.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &T) -> Result<usize, EncodeError> {
        String.compute_size(Some(value))
    }
}

impl Decoder<Option<StdString>> for String {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<StdString>, DecodeError> {
        match Int16.decode(buf)? {
            -1 => Ok(None),
            n if n >= 0 => {
                let mut strbuf = vec![0; n as usize];
                buf.try_copy_to_slice(&mut strbuf)?;
                Ok(Some(std::string::String::from_utf8(strbuf)?))
            }
            n => {
                error!("String length is negative ({})", n);
                Err(DecodeError)
            }
        }
    }
}

impl<T: NewType<StrBytes>> Decoder<Option<T>> for String {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<T>, DecodeError> {
        match Int16.decode(buf)? {
            -1 => Ok(None),
            n if n >= 0 => {
                let strbuf = StrBytes::try_from(buf.try_get_bytes(n as usize)?)?;
                Ok(Some(strbuf.into()))
            }
            n => {
                error!("String length is negative ({})", n);
                Err(DecodeError)
            }
        }
    }
}

impl Decoder<StdString> for String {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<StdString, DecodeError> {
        match String.decode(buf) {
            Ok(None) => {
                error!("String length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

impl<T: NewType<StrBytes>> Decoder<T> for String {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<T, DecodeError> {
        match String.decode(buf) {
            Ok(None) => {
                error!("String length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

/// A string whose length is encoded with a `u32` or varint.
#[derive(Debug, Copy, Clone, Default)]
pub struct CompactString;

impl Encoder<Option<&str>> for CompactString {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: Option<&str>) -> Result<(), EncodeError> {
        if let Some(s) = value {
            // Use >= because we're going to add one to the length
            if s.len() >= std::u32::MAX as usize {
                error!("CompactString is too long to encode ({} bytes)", s.len());
                Err(EncodeError)
            } else {
                UnsignedVarInt.encode(buf, (s.len() as u32) + 1)?;
                buf.put_slice(s.as_bytes());
                Ok(())
            }
        } else {
            UnsignedVarInt.encode(buf, 0)?;
            Ok(())
        }
    }
    fn compute_size(&self, value: Option<&str>) -> Result<usize, EncodeError> {
        if let Some(s) = value {
            // Use >= because we're going to add one to the length
            if s.len() >= std::u32::MAX as usize {
                error!("CompactString is too long to encode ({} bytes)", s.len());
                Err(EncodeError)
            } else {
                Ok(UnsignedVarInt.compute_size((s.len() as u32) + 1)? + s.len())
            }
        } else {
            Ok(1)
        }
    }
}

impl Encoder<Option<&StdString>> for CompactString {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: Option<&StdString>,
    ) -> Result<(), EncodeError> {
        CompactString.encode(buf, value.map(|s| s.as_str()))
    }
    fn compute_size(&self, value: Option<&StdString>) -> Result<usize, EncodeError> {
        CompactString.compute_size(value.map(|s| s.as_str()))
    }
}

impl<T: NewType<StrBytes>> Encoder<Option<&T>> for CompactString {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: Option<&T>) -> Result<(), EncodeError> {
        CompactString.encode(buf, value.map(|s| &**s.borrow()))
    }
    fn compute_size(&self, value: Option<&T>) -> Result<usize, EncodeError> {
        CompactString.compute_size(value.map(|s| &**s.borrow()))
    }
}

impl Encoder<&Option<StdString>> for CompactString {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &Option<StdString>,
    ) -> Result<(), EncodeError> {
        CompactString.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<StdString>) -> Result<usize, EncodeError> {
        CompactString.compute_size(value.as_ref())
    }
}

impl<T: NewType<StrBytes>> Encoder<&Option<T>> for CompactString {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &Option<T>) -> Result<(), EncodeError> {
        CompactString.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<T>) -> Result<usize, EncodeError> {
        CompactString.compute_size(value.as_ref())
    }
}

impl Encoder<&str> for CompactString {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &str) -> Result<(), EncodeError> {
        CompactString.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &str) -> Result<usize, EncodeError> {
        CompactString.compute_size(Some(value))
    }
}

impl Encoder<&StdString> for CompactString {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &StdString) -> Result<(), EncodeError> {
        CompactString.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &StdString) -> Result<usize, EncodeError> {
        CompactString.compute_size(Some(value))
    }
}

impl<T: NewType<StrBytes>> Encoder<&T> for CompactString {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &T) -> Result<(), EncodeError> {
        CompactString.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &T) -> Result<usize, EncodeError> {
        CompactString.compute_size(Some(value))
    }
}

impl Decoder<Option<StdString>> for CompactString {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<StdString>, DecodeError> {
        match UnsignedVarInt.decode(buf)? {
            0 => Ok(None),
            n => {
                let mut strbuf = vec![0; (n - 1) as usize];
                buf.try_copy_to_slice(&mut strbuf)?;
                Ok(Some(std::string::String::from_utf8(strbuf)?))
            }
        }
    }
}

impl<T: NewType<StrBytes>> Decoder<Option<T>> for CompactString {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<T>, DecodeError> {
        match UnsignedVarInt.decode(buf)? {
            0 => Ok(None),
            n => {
                let strbuf = StrBytes::try_from(buf.try_get_bytes((n - 1) as usize)?)?;
                Ok(Some(strbuf.into()))
            }
        }
    }
}

impl Decoder<StdString> for CompactString {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<StdString, DecodeError> {
        match CompactString.decode(buf) {
            Ok(None) => {
                error!("CompactString length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

impl<T: NewType<StrBytes>> Decoder<T> for CompactString {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<T, DecodeError> {
        match CompactString.decode(buf) {
            Ok(None) => {
                error!("CompactString length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

/// A sequence of bytes, up to [`i32::MAX`] long.
#[derive(Debug, Copy, Clone, Default)]
pub struct Bytes;

impl Encoder<Option<&[u8]>> for Bytes {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: Option<&[u8]>) -> Result<(), EncodeError> {
        if let Some(s) = value {
            if s.len() > i32::MAX as usize {
                error!("Data is too long to encode ({} bytes)", s.len());
                Err(EncodeError)
            } else {
                Int32.encode(buf, s.len() as i32)?;
                buf.put_slice(s);
                Ok(())
            }
        } else {
            Int32.encode(buf, -1)?;
            Ok(())
        }
    }
    fn compute_size(&self, value: Option<&[u8]>) -> Result<usize, EncodeError> {
        if let Some(s) = value {
            if s.len() > i32::MAX as usize {
                error!("Data is too long to encode ({} bytes)", s.len());
                Err(EncodeError)
            } else {
                Ok(4 + s.len())
            }
        } else {
            Ok(4)
        }
    }
}

impl Encoder<Option<&Vec<u8>>> for Bytes {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: Option<&Vec<u8>>,
    ) -> Result<(), EncodeError> {
        Bytes.encode(buf, value.map(|s| s.as_slice()))
    }
    fn compute_size(&self, value: Option<&Vec<u8>>) -> Result<usize, EncodeError> {
        Bytes.compute_size(value.map(|s| s.as_slice()))
    }
}

impl Encoder<&Option<Vec<u8>>> for Bytes {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &Option<Vec<u8>>,
    ) -> Result<(), EncodeError> {
        Bytes.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<Vec<u8>>) -> Result<usize, EncodeError> {
        Bytes.compute_size(value.as_ref())
    }
}

impl Encoder<Option<&bytes::Bytes>> for Bytes {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: Option<&bytes::Bytes>,
    ) -> Result<(), EncodeError> {
        Bytes.encode(buf, value.map(|s| &**s))
    }
    fn compute_size(&self, value: Option<&bytes::Bytes>) -> Result<usize, EncodeError> {
        Bytes.compute_size(value.map(|s| &**s))
    }
}

impl Encoder<&Option<bytes::Bytes>> for Bytes {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &Option<bytes::Bytes>,
    ) -> Result<(), EncodeError> {
        Bytes.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<bytes::Bytes>) -> Result<usize, EncodeError> {
        Bytes.compute_size(value.as_ref())
    }
}

impl Encoder<&[u8]> for Bytes {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &[u8]) -> Result<(), EncodeError> {
        Bytes.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &[u8]) -> Result<usize, EncodeError> {
        Bytes.compute_size(Some(value))
    }
}

impl Encoder<&Vec<u8>> for Bytes {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &Vec<u8>) -> Result<(), EncodeError> {
        Bytes.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &Vec<u8>) -> Result<usize, EncodeError> {
        Bytes.compute_size(Some(value))
    }
}

impl Encoder<&bytes::Bytes> for Bytes {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &bytes::Bytes) -> Result<(), EncodeError> {
        Bytes.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &bytes::Bytes) -> Result<usize, EncodeError> {
        Bytes.compute_size(Some(value))
    }
}

impl Decoder<Option<Vec<u8>>> for Bytes {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<Vec<u8>>, DecodeError> {
        match Int32.decode(buf)? {
            -1 => Ok(None),
            n if n >= 0 => {
                let mut data = vec![0; n as usize];
                buf.try_copy_to_slice(&mut data)?;
                Ok(Some(data))
            }
            n => {
                error!("Data length is negative ({})", n);
                Err(DecodeError)
            }
        }
    }
}

impl Decoder<Option<bytes::Bytes>> for Bytes {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<bytes::Bytes>, DecodeError> {
        match Int32.decode(buf)? {
            -1 => Ok(None),
            n if n >= 0 => Ok(Some(buf.try_get_bytes(n as usize)?)),
            n => {
                error!("Data length is negative ({})", n);
                Err(DecodeError)
            }
        }
    }
}

impl Decoder<Vec<u8>> for Bytes {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Vec<u8>, DecodeError> {
        match Bytes.decode(buf) {
            Ok(None) => {
                error!("Data length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

impl Decoder<bytes::Bytes> for Bytes {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<bytes::Bytes, DecodeError> {
        match Bytes.decode(buf) {
            Ok(None) => {
                error!("Data length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

/// A sequence of bytes that is encoded with a `u32` or a varint, depending on size.
#[derive(Debug, Copy, Clone, Default)]
pub struct CompactBytes;

impl Encoder<Option<&[u8]>> for CompactBytes {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: Option<&[u8]>) -> Result<(), EncodeError> {
        if let Some(s) = value {
            // Use >= because we're going to add one to the length
            if s.len() >= std::u32::MAX as usize {
                error!("CompactBytes is too long to encode ({} bytes)", s.len());
                Err(EncodeError)
            } else {
                UnsignedVarInt.encode(buf, (s.len() as u32) + 1)?;
                buf.put_slice(s);
                Ok(())
            }
        } else {
            UnsignedVarInt.encode(buf, 0)?;
            Ok(())
        }
    }
    fn compute_size(&self, value: Option<&[u8]>) -> Result<usize, EncodeError> {
        if let Some(s) = value {
            // Use >= because we're going to add one to the length
            if s.len() >= std::u32::MAX as usize {
                error!("CompactBytes is too long to encode ({} bytes)", s.len());
                Err(EncodeError)
            } else {
                Ok(UnsignedVarInt.compute_size((s.len() as u32) + 1)? + s.len())
            }
        } else {
            Ok(1)
        }
    }
}

impl Encoder<Option<&Vec<u8>>> for CompactBytes {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: Option<&Vec<u8>>,
    ) -> Result<(), EncodeError> {
        CompactBytes.encode(buf, value.map(|s| s.as_slice()))
    }
    fn compute_size(&self, value: Option<&Vec<u8>>) -> Result<usize, EncodeError> {
        CompactBytes.compute_size(value.map(|s| s.as_slice()))
    }
}

impl Encoder<&Option<Vec<u8>>> for CompactBytes {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &Option<Vec<u8>>,
    ) -> Result<(), EncodeError> {
        CompactBytes.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<Vec<u8>>) -> Result<usize, EncodeError> {
        CompactBytes.compute_size(value.as_ref())
    }
}

impl Encoder<Option<&bytes::Bytes>> for CompactBytes {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: Option<&bytes::Bytes>,
    ) -> Result<(), EncodeError> {
        CompactBytes.encode(buf, value.map(|s| &**s))
    }
    fn compute_size(&self, value: Option<&bytes::Bytes>) -> Result<usize, EncodeError> {
        CompactBytes.compute_size(value.map(|s| &**s))
    }
}

impl Encoder<&Option<bytes::Bytes>> for CompactBytes {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &Option<bytes::Bytes>,
    ) -> Result<(), EncodeError> {
        CompactBytes.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<bytes::Bytes>) -> Result<usize, EncodeError> {
        CompactBytes.compute_size(value.as_ref())
    }
}

impl Encoder<&[u8]> for CompactBytes {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &[u8]) -> Result<(), EncodeError> {
        CompactBytes.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &[u8]) -> Result<usize, EncodeError> {
        CompactBytes.compute_size(Some(value))
    }
}

impl Encoder<&Vec<u8>> for CompactBytes {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &Vec<u8>) -> Result<(), EncodeError> {
        CompactBytes.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &Vec<u8>) -> Result<usize, EncodeError> {
        CompactBytes.compute_size(Some(value))
    }
}

impl Encoder<&bytes::Bytes> for CompactBytes {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &bytes::Bytes) -> Result<(), EncodeError> {
        CompactBytes.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &bytes::Bytes) -> Result<usize, EncodeError> {
        CompactBytes.compute_size(Some(value))
    }
}

impl Decoder<Option<Vec<u8>>> for CompactBytes {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<Vec<u8>>, DecodeError> {
        match UnsignedVarInt.decode(buf)? {
            0 => Ok(None),
            n => {
                let mut data = vec![0; (n - 1) as usize];
                buf.try_copy_to_slice(&mut data)?;
                Ok(Some(data))
            }
        }
    }
}

impl Decoder<Option<bytes::Bytes>> for CompactBytes {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<bytes::Bytes>, DecodeError> {
        match UnsignedVarInt.decode(buf)? {
            0 => Ok(None),
            n => Ok(Some(buf.try_get_bytes((n - 1) as usize)?)),
        }
    }
}

impl Decoder<Vec<u8>> for CompactBytes {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Vec<u8>, DecodeError> {
        match CompactBytes.decode(buf) {
            Ok(None) => {
                error!("Data length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

impl Decoder<bytes::Bytes> for CompactBytes {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<bytes::Bytes, DecodeError> {
        match CompactBytes.decode(buf) {
            Ok(None) => {
                error!("Data length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

/// A struct, which is encoded according to the type it represents.
#[derive(Debug, Copy, Clone, Default)]
pub struct Struct {
    /// The version of the struct.
    pub version: i16,
}

impl<T: Encodable> Encoder<&T> for Struct {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &T) -> Result<(), EncodeError> {
        value.encode(buf, self.version)
    }
    fn compute_size(&self, value: &T) -> Result<usize, EncodeError> {
        value.compute_size(self.version)
    }
}

impl<T: Encodable> Encoder<&Option<T>> for Struct {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, option: &Option<T>) -> Result<(), EncodeError> {
        if let Some(value) = option {
            // this one byte represents optionality (1 for present)
            Int8.encode(buf, 1)?;
            value.encode(buf, self.version)?;
            Ok(())
        } else {
            // this one byte represents optionality (-1 for missing)
            Int8.encode(buf, -1)?;
            Ok(())
        }
        
    }
    fn compute_size(&self, option: &Option<T>) -> Result<usize, EncodeError> {
        if let Some(value) = option {
            // +1 to size is the one additional byte that represents optionality of struct (value -1 for missing and 1 for present)
            Ok(value.compute_size(self.version)? + 1)
        } else {
            // one byte that represents optionality (value -1 for missing and 1 for present)
            Ok(1)
        }
    }
}

impl<T: Decodable> Decoder<T> for Struct {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<T, DecodeError> {
        T::decode(buf, self.version)
    }
}

impl<T: Decodable> Decoder<Option<T>> for Struct {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<T>, DecodeError> {
        // first i8 is optionality of struct (value -1 for missing and 1 for present)
        let optional: i8 = Int8.decode(buf)?;
        if optional == -1 {
            Ok(None)
        } else {
            let decoded = T::decode(buf, self.version)?;
            Ok(Some(decoded))
        }
    }
}

impl<T: MapEncodable> Encoder<(&T::Key, &T)> for Struct {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        (key, value): (&T::Key, &T),
    ) -> Result<(), EncodeError> {
        value.encode(key, buf, self.version)
    }
    fn compute_size(&self, (key, value): (&T::Key, &T)) -> Result<usize, EncodeError> {
        value.compute_size(key, self.version)
    }
}

impl<T: MapDecodable> Decoder<(T::Key, T)> for Struct {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<(T::Key, T), DecodeError> {
        T::decode(buf, self.version)
    }
}

/// An array whose length is encoded with an `i32`.
#[derive(Debug, Copy, Clone)]
pub struct Array<E>(pub E);

impl<T, E: for<'a> Encoder<&'a T>> Encoder<Option<&[T]>> for Array<E> {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: Option<&[T]>) -> Result<(), EncodeError> {
        if let Some(a) = value {
            if a.len() > i32::MAX as usize {
                error!("Array is too long to encode ({} items)", a.len());
                Err(EncodeError)
            } else {
                Int32.encode(buf, a.len() as i32)?;
                for item in a {
                    self.0.encode(buf, item)?;
                }
                Ok(())
            }
        } else {
            Int32.encode(buf, -1)?;
            Ok(())
        }
    }
    fn compute_size(&self, value: Option<&[T]>) -> Result<usize, EncodeError> {
        if let Some(a) = value {
            if a.len() > i32::MAX as usize {
                error!("Array is too long to encode ({} items)", a.len());
                Err(EncodeError)
            } else if let Some(fixed_size) = self.0.fixed_size() {
                Ok(4 + a.len() * fixed_size)
            } else {
                let mut total_size = 4;
                for item in a {
                    total_size += self.0.compute_size(item)?;
                }
                Ok(total_size)
            }
        } else {
            Ok(4)
        }
    }
}

impl<T, E: for<'a> Encoder<&'a T>> Encoder<Option<&Vec<T>>> for Array<E> {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: Option<&Vec<T>>,
    ) -> Result<(), EncodeError> {
        self.encode(buf, value.map(|s| s.as_slice()))
    }
    fn compute_size(&self, value: Option<&Vec<T>>) -> Result<usize, EncodeError> {
        self.compute_size(value.map(|s| s.as_slice()))
    }
}

impl<T, E: for<'a> Encoder<&'a T>> Encoder<&Option<Vec<T>>> for Array<E> {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &Option<Vec<T>>,
    ) -> Result<(), EncodeError> {
        self.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<Vec<T>>) -> Result<usize, EncodeError> {
        self.compute_size(value.as_ref())
    }
}

impl<T, E: for<'a> Encoder<&'a T>> Encoder<&[T]> for Array<E> {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &[T]) -> Result<(), EncodeError> {
        self.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &[T]) -> Result<usize, EncodeError> {
        self.compute_size(Some(value))
    }
}

impl<T, E: for<'a> Encoder<&'a T>> Encoder<&Vec<T>> for Array<E> {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &Vec<T>) -> Result<(), EncodeError> {
        self.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &Vec<T>) -> Result<usize, EncodeError> {
        self.compute_size(Some(value))
    }
}

impl<K: Eq + Hash, V, E: for<'a> Encoder<(&'a K, &'a V)>> Encoder<Option<&IndexMap<K, V>>>
    for Array<E>
{
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: Option<&IndexMap<K, V>>,
    ) -> Result<(), EncodeError> {
        if let Some(a) = value {
            if a.len() > i32::MAX as usize {
                error!("Array is too long to encode ({} items)", a.len());
                Err(EncodeError)
            } else {
                Int32.encode(buf, a.len() as i32)?;
                for item in a {
                    self.0.encode(buf, item)?;
                }
                Ok(())
            }
        } else {
            Int32.encode(buf, -1)?;
            Ok(())
        }
    }
    fn compute_size(&self, value: Option<&IndexMap<K, V>>) -> Result<usize, EncodeError> {
        if let Some(a) = value {
            if a.len() > i32::MAX as usize {
                error!("Array is too long to encode ({} items)", a.len());
                Err(EncodeError)
            } else if let Some(fixed_size) = self.0.fixed_size() {
                Ok(4 + a.len() * fixed_size)
            } else {
                let mut total_size = 4;
                for item in a {
                    total_size += self.0.compute_size(item)?;
                }
                Ok(total_size)
            }
        } else {
            Ok(4)
        }
    }
}

impl<K: Eq + Hash, V, E: for<'a> Encoder<(&'a K, &'a V)>> Encoder<&Option<IndexMap<K, V>>>
    for Array<E>
{
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &Option<IndexMap<K, V>>,
    ) -> Result<(), EncodeError> {
        self.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<IndexMap<K, V>>) -> Result<usize, EncodeError> {
        self.compute_size(value.as_ref())
    }
}

impl<K: Eq + Hash, V, E: for<'a> Encoder<(&'a K, &'a V)>> Encoder<&IndexMap<K, V>> for Array<E> {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &IndexMap<K, V>,
    ) -> Result<(), EncodeError> {
        self.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &IndexMap<K, V>) -> Result<usize, EncodeError> {
        self.compute_size(Some(value))
    }
}

impl<T, E: Decoder<T>> Decoder<Option<Vec<T>>> for Array<E> {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<Vec<T>>, DecodeError> {
        match Int32.decode(buf)? {
            -1 => Ok(None),
            n if n >= 0 => {
                let mut result = Vec::with_capacity(n as usize);
                for _ in 0..n {
                    result.push(self.0.decode(buf)?);
                }
                Ok(Some(result))
            }
            n => {
                error!("Array length is negative ({})", n);
                Err(DecodeError)
            }
        }
    }
}

impl<T, E: Decoder<T>> Decoder<Vec<T>> for Array<E> {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Vec<T>, DecodeError> {
        match self.decode(buf) {
            Ok(None) => {
                error!("Array length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

impl<K: Eq + Hash, V, E: Decoder<(K, V)>> Decoder<Option<IndexMap<K, V>>> for Array<E> {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<IndexMap<K, V>>, DecodeError> {
        match Int32.decode(buf)? {
            -1 => Ok(None),
            n if n >= 0 => {
                let mut result = IndexMap::new();
                for _ in 0..n {
                    let (k, v) = self.0.decode(buf)?;
                    result.insert(k, v);
                }
                Ok(Some(result))
            }
            n => {
                error!("Array length is negative ({})", n);
                Err(DecodeError)
            }
        }
    }
}

impl<K: Eq + Hash, V, E: Decoder<(K, V)>> Decoder<IndexMap<K, V>> for Array<E> {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<IndexMap<K, V>, DecodeError> {
        match self.decode(buf) {
            Ok(None) => {
                error!("Array length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

/// An array whose length is encoded with a varint.
#[derive(Debug, Copy, Clone)]
pub struct CompactArray<E>(pub E);

impl<T, E: for<'a> Encoder<&'a T>> Encoder<Option<&[T]>> for CompactArray<E> {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: Option<&[T]>) -> Result<(), EncodeError> {
        if let Some(a) = value {
            // Use >= because we're going to add one to the length
            if a.len() >= std::u32::MAX as usize {
                error!("CompactArray is too long to encode ({} items)", a.len());
                Err(EncodeError)
            } else {
                UnsignedVarInt.encode(buf, (a.len() as u32) + 1)?;
                for item in a {
                    self.0.encode(buf, item)?;
                }
                Ok(())
            }
        } else {
            UnsignedVarInt.encode(buf, 0)?;
            Ok(())
        }
    }
    fn compute_size(&self, value: Option<&[T]>) -> Result<usize, EncodeError> {
        if let Some(a) = value {
            // Use >= because we're going to add one to the length
            if a.len() >= std::u32::MAX as usize {
                error!("CompactArray is too long to encode ({} items)", a.len());
                Err(EncodeError)
            } else if let Some(fixed_size) = self.0.fixed_size() {
                Ok(UnsignedVarInt.compute_size((a.len() as u32) + 1)? + a.len() * fixed_size)
            } else {
                let mut total_size = UnsignedVarInt.compute_size((a.len() as u32) + 1)?;
                for item in a {
                    total_size += self.0.compute_size(item)?;
                }
                Ok(total_size)
            }
        } else {
            Ok(1)
        }
    }
}

impl<T, E: for<'a> Encoder<&'a T>> Encoder<Option<&Vec<T>>> for CompactArray<E> {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: Option<&Vec<T>>,
    ) -> Result<(), EncodeError> {
        self.encode(buf, value.map(|s| s.as_slice()))
    }
    fn compute_size(&self, value: Option<&Vec<T>>) -> Result<usize, EncodeError> {
        self.compute_size(value.map(|s| s.as_slice()))
    }
}

impl<T, E: for<'a> Encoder<&'a T>> Encoder<&Option<Vec<T>>> for CompactArray<E> {
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &Option<Vec<T>>,
    ) -> Result<(), EncodeError> {
        self.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<Vec<T>>) -> Result<usize, EncodeError> {
        self.compute_size(value.as_ref())
    }
}

impl<T, E: for<'a> Encoder<&'a T>> Encoder<&[T]> for CompactArray<E> {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &[T]) -> Result<(), EncodeError> {
        self.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &[T]) -> Result<usize, EncodeError> {
        self.compute_size(Some(value))
    }
}

impl<T, E: for<'a> Encoder<&'a T>> Encoder<&Vec<T>> for CompactArray<E> {
    fn encode<B: ByteBufMut>(&self, buf: &mut B, value: &Vec<T>) -> Result<(), EncodeError> {
        self.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &Vec<T>) -> Result<usize, EncodeError> {
        self.compute_size(Some(value))
    }
}

impl<K: Eq + Hash, V, E: for<'a> Encoder<(&'a K, &'a V)>> Encoder<Option<&IndexMap<K, V>>>
    for CompactArray<E>
{
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: Option<&IndexMap<K, V>>,
    ) -> Result<(), EncodeError> {
        if let Some(a) = value {
            // Use >= because we're going to add one to the length
            if a.len() >= std::u32::MAX as usize {
                error!("CompactArray is too long to encode ({} items)", a.len());
                Err(EncodeError)
            } else {
                UnsignedVarInt.encode(buf, (a.len() as u32) + 1)?;
                for item in a {
                    self.0.encode(buf, item)?;
                }
                Ok(())
            }
        } else {
            UnsignedVarInt.encode(buf, 0)?;
            Ok(())
        }
    }
    fn compute_size(&self, value: Option<&IndexMap<K, V>>) -> Result<usize, EncodeError> {
        if let Some(a) = value {
            // Use >= because we're going to add one to the length
            if a.len() >= std::u32::MAX as usize {
                error!("CompactArray is too long to encode ({} items)", a.len());
                Err(EncodeError)
            } else if let Some(fixed_size) = self.0.fixed_size() {
                Ok(UnsignedVarInt.compute_size((a.len() as u32) + 1)? + a.len() * fixed_size)
            } else {
                let mut total_size = UnsignedVarInt.compute_size((a.len() as u32) + 1)?;
                for item in a {
                    total_size += self.0.compute_size(item)?;
                }
                Ok(total_size)
            }
        } else {
            Ok(1)
        }
    }
}

impl<K: Eq + Hash, V, E: for<'a> Encoder<(&'a K, &'a V)>> Encoder<&Option<IndexMap<K, V>>>
    for CompactArray<E>
{
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &Option<IndexMap<K, V>>,
    ) -> Result<(), EncodeError> {
        self.encode(buf, value.as_ref())
    }
    fn compute_size(&self, value: &Option<IndexMap<K, V>>) -> Result<usize, EncodeError> {
        self.compute_size(value.as_ref())
    }
}

impl<K: Eq + Hash, V, E: for<'a> Encoder<(&'a K, &'a V)>> Encoder<&IndexMap<K, V>>
    for CompactArray<E>
{
    fn encode<B: ByteBufMut>(
        &self,
        buf: &mut B,
        value: &IndexMap<K, V>,
    ) -> Result<(), EncodeError> {
        self.encode(buf, Some(value))
    }
    fn compute_size(&self, value: &IndexMap<K, V>) -> Result<usize, EncodeError> {
        self.compute_size(Some(value))
    }
}

impl<T, E: Decoder<T>> Decoder<Option<Vec<T>>> for CompactArray<E> {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<Vec<T>>, DecodeError> {
        match UnsignedVarInt.decode(buf)? {
            0 => Ok(None),
            n => {
                let mut result = Vec::with_capacity((n - 1) as usize);
                for _ in 1..n {
                    result.push(self.0.decode(buf)?);
                }
                Ok(Some(result))
            }
        }
    }
}

impl<T, E: Decoder<T>> Decoder<Vec<T>> for CompactArray<E> {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Vec<T>, DecodeError> {
        match self.decode(buf) {
            Ok(None) => {
                error!("CompactArray length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

impl<K: Eq + Hash, V, E: Decoder<(K, V)>> Decoder<Option<IndexMap<K, V>>> for CompactArray<E> {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<Option<IndexMap<K, V>>, DecodeError> {
        match UnsignedVarInt.decode(buf)? {
            0 => Ok(None),
            n => {
                let mut result = IndexMap::new();
                for _ in 1..n {
                    let (k, v) = self.0.decode(buf)?;
                    result.insert(k, v);
                }
                Ok(Some(result))
            }
        }
    }
}

impl<K: Eq + Hash, V, E: Decoder<(K, V)>> Decoder<IndexMap<K, V>> for CompactArray<E> {
    fn decode<B: ByteBuf>(&self, buf: &mut B) -> Result<IndexMap<K, V>, DecodeError> {
        match self.decode(buf) {
            Ok(None) => {
                error!("CompactArray length is negative (-1)");
                Err(DecodeError)
            }
            Ok(Some(s)) => Ok(s),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fmt::Debug;

    fn test_encoder_decoder<V: PartialEq + Debug, E: for<'a> Encoder<&'a V> + Decoder<V>>(
        encoder: E,
        value: V,
        mut expected: &[u8],
    ) {
        let mut buf = vec![];
        encoder.encode(&mut buf, &value).unwrap();
        assert_eq!(buf, expected);

        let decoded = encoder.decode(&mut expected).unwrap();
        assert_eq!(value, decoded);
    }

    #[test]
    fn smoke_varint_encoder_decoder() {
        test_encoder_decoder(VarInt, 0, &[0]);
        test_encoder_decoder(VarInt, -1, &[1]);
        test_encoder_decoder(VarInt, 1, &[2]);
        test_encoder_decoder(VarInt, -2, &[3]);
        test_encoder_decoder(VarInt, 300, &[216, 4]);
        test_encoder_decoder(VarInt, i32::MAX, &[254, 255, 255, 255, 15]);
        test_encoder_decoder(VarInt, i32::MIN, &[255, 255, 255, 255, 15]);
    }

    #[test]
    fn smoke_varlong_encoder_decoder() {
        test_encoder_decoder(VarLong, 0, &[0]);
        test_encoder_decoder(VarLong, -1, &[1]);
        test_encoder_decoder(VarLong, 1, &[2]);
        test_encoder_decoder(VarLong, -2, &[3]);
        test_encoder_decoder(VarLong, 300, &[216, 4]);
        test_encoder_decoder(
            VarLong,
            std::i64::MAX,
            &[254, 255, 255, 255, 255, 255, 255, 255, 255, 1],
        );
        test_encoder_decoder(
            VarLong,
            std::i64::MIN,
            &[255, 255, 255, 255, 255, 255, 255, 255, 255, 1],
        );
    }

    #[test]
    fn smoke_string_encoder_decoder() {
        test_encoder_decoder(
            String,
            std::string::String::from("hello"),
            &[0, 5, 104, 101, 108, 108, 111],
        );
        test_encoder_decoder(String, None::<std::string::String>, &[255, 255]);
    }

    #[test]
    fn smoke_compact_string_encoder_decoder() {
        test_encoder_decoder(
            CompactString,
            std::string::String::from("hello"),
            &[6, 104, 101, 108, 108, 111],
        );
        test_encoder_decoder(CompactString, None::<std::string::String>, &[0]);
    }

    #[test]
    fn smoke_bytes_encoder_decoder() {
        test_encoder_decoder(Bytes, vec![1, 2, 3, 4], &[0, 0, 0, 4, 1, 2, 3, 4]);
        test_encoder_decoder(Bytes, None::<Vec<u8>>, &[255, 255, 255, 255]);
    }

    #[test]
    fn smoke_compact_bytes_encoder_decoder() {
        test_encoder_decoder(CompactBytes, vec![1, 2, 3, 4], &[5, 1, 2, 3, 4]);
        test_encoder_decoder(CompactBytes, None::<Vec<u8>>, &[0]);
    }
}
