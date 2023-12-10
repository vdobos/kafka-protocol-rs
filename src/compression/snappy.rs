use std::cmp::min;
use std::str::from_utf8;

use bytes::{Bytes, BytesMut, Buf};
use log::error;
use snap::raw::*;

use crate::protocol::buf::{ByteBuf, ByteBufMut};
use crate::protocol::{DecodeError, EncodeError};

use super::{Compressor, Decompressor};

/// Snappy compression algorithm. See [Kafka's broker configuration](https://kafka.apache.org/documentation/#brokerconfigs_compression.type)
/// for more information.
/// 
/// Uses custom framing and header used by SnappyOutputStream.java implementation in xerial snappy-java library
/// https://github.com/xerial/snappy-java that is used for compression in de-facto reference Java kafka client
/// More info can be found -> https://github.com/dpkp/kafka-python/issues/126#issuecomment-35478921
/// (TL;DR xerial snappy has its own "speshul" take on framed snappy compression incompatible with rest of the known universe)
/// 
/// Code for xerial snappy seen here was adapted from
/// https://github.com/eapache/go-xerial-snappy/blob/master/snappy.go
/// 
/// Alternative golang implementation (used by non-confluent golang kafka clients), mentioned for completness:
/// https://github.com/klauspost/compress/tree/master/snappy/xerial
pub struct Snappy;

// xerial snappy-java/java kafka client compatible snappy compression
// adapted from golang code -> https://github.com/eapache/go-xerial-snappy/blob/master/snappy.go
// java client/consumer can easily read "normal" compressed records created by simple use of Encoder::new(), as was originally also 
// produced by this implementation, but it's better to use xerial for consumers that might expect only output from "reference" java client library
impl<B: ByteBufMut> Compressor<B> for Snappy {
    type BufMut = BytesMut;
    fn compress<R, F>(buf: &mut B, f: F) -> Result<R, EncodeError>
    where
        F: FnOnce(&mut Self::BufMut) -> Result<R, EncodeError>,
    {
        // Write uncompressed bytes into a temporary buffer
        let mut tmp = BytesMut::new();
        let res = f(&mut tmp)?;

        // xerial header bytes
        let header: [u8; 16] = [130, 83, 78, 65, 80, 80, 89, 0, 0, 0, 0, 1, 0, 0, 0, 1];
        buf.put_slice(header.as_slice());

        let max = tmp.len();
        let block_size: usize = 32 * 1024;
        let mut pos: usize = 0;
        let mut encoder = Encoder::new();

        while pos < max {
            let new_pos = min(pos + block_size, max);
            
            let chunk = &tmp[pos..new_pos];
            let max_len = max_compress_len(chunk.len());
            let mut chunk_buffer: Vec<u8> = vec![0; max_len];

            let actual_len = encoder.compress(chunk, &mut chunk_buffer).map_err(|e| {
                error!("Failed to compress buffer: {}", e);
                EncodeError
            })?;
            buf.put_u32(actual_len as u32);
            buf.put_slice(&chunk_buffer[..actual_len]);

            pos = new_pos;
        };

        Ok(res)
    }
}

// xerial snappy-java/java kafka client compatible snappy decompression
// adapted from golang code -> https://github.com/eapache/go-xerial-snappy/blob/master/snappy.go
impl<B: ByteBuf> Decompressor<B> for Snappy {
    type Buf = Bytes;
    fn decompress<R, F>(buf: &mut B, f: F) -> Result<R, DecodeError>
    where
        F: FnOnce(&mut Self::Buf) -> Result<R, DecodeError>,
    {
        // Allocate a temporary buffer to hold the uncompressed bytes
        let buf = buf.copy_to_bytes(buf.remaining());

        // detecting xerial header to also support normal snappy just-in-case
        let is_from_xerial = buf.len() >= 16 && buf[0] == 130 && from_utf8(&buf[1..7]).unwrap() == "SNAPPY";
        if is_from_xerial {
            let mut res: Vec<u8> = Vec::new();
            let mut decoder = Decoder::new();

            let mut pos = 0;
            let max = buf.len();
            // xerial special header is 16 bytes
            pos += 16;

            while (pos + 4) <= max {
                let size = (&buf[pos..(pos + 4)]).try_get_u32().map_err(|e| {
                    error!("Failed to decompress buffer: {}", e);
                    DecodeError
                })?;
                pos += 4;

                let next_pos = pos + (size as usize);

                // this deals with possible overflows on 32 bit systems (copied from golang original)
                if next_pos < pos || next_pos > max {
                    error!("Frame cursor overflow");

                    return Err(DecodeError);
                };

                let chunk = &buf[pos..next_pos];
                let len = decompress_len(chunk).map_err(|e| {
                    error!("Failed to decompress buffer: {}", e);
                    DecodeError
                })?;
                let mut chunk_buffer: Vec<u8> = vec![0; len];

                let _ = decoder.decompress(chunk, &mut chunk_buffer);

                pos = next_pos;
                res.append(&mut chunk_buffer);
            };

            f(&mut res.into())
        } else {
            let actual_len = decompress_len(&buf).map_err(|e| {
                error!("Failed to decompress buffer: {}", e);
                DecodeError
            })?;
            let mut tmp = BytesMut::new();
            tmp.resize(actual_len, 0);
            
            // Decompress directly from the input buffer
            Decoder::new().decompress(&buf, &mut tmp).map_err(|e| {
                error!("Failed to decompress buffer: {}", e);
                DecodeError
            })?;

            f(&mut tmp.into())
        }
    }
}