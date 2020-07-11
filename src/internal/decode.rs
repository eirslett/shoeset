use internal::ArchiveError;
use std::io;
use std::io::Read;
// use internal::encoded_header::decompress_entry;
/*
use bytes::buf::{BufExt, BufMutExt};
use bytes::{Buf, BufMut, Bytes};
*/


fn decode_lzma(reader: &[u8], properties: &[u8], unpack_size: u64) -> Result<Vec<u8>, ArchiveError> {
    /*
    let mut props_buffer = buffer::Buffer::new(&properties);
    let d = props_buffer.read();
    let dictionary = read_utils::read_uint32(&mut props_buffer);

    let lc = d % 9;
    let d  = d / 9;
    let pb = d / 5;
    let lp = d % 5;

    let props = lzma::properties::Properties {
        /// Literal context bits.
        lc,

        /// Literal position bits.
        lp,

        /// Position bits.
        pb,

        /// Dictionary size.
        dictionary,

        /// Uncompressed size if present.
        uncompressed: Some(unpack_size),
    };

    let cursor = io::Cursor::new(&reader);
    let stream = lzma::reader::Reader::new(cursor, props).map_err(|e| ArchiveError::new(&e.to_string()))?;
    let a = read_n(stream, unpack_size);
    */

    // let mut cursor = io::Cursor:new(&reader);
    let mut cursor = properties.chain(reader);
    // let mut out = vec![0u8;5];
    let mut out = Vec::with_capacity(unpack_size as usize); // vec![0u8; unpack_size as usize];
    // lzma_rs::lzma_decompress(&mut cursor, &mut out);

    let unpacked_size = lzma_rs::decompress::UnpackedSize::UseProvided(Some(unpack_size));
    lzma_rs::lzma_decompress_with_options(&mut cursor, &mut out, &lzma_rs::decompress::Options { unpacked_size }).map_err(|e| ArchiveError::new(&format!("{:?}", e)))?;

    let b = out;

    // println!("A {}: {:?}", a.len(), a);
    // println!("B {}: {:?}", b.len(), b);
    // println!("Properties: {:?}", properties);
    // panic!("hey");
    Ok(b)

}

fn decode_lzma2(reader: &[u8], _properties: &[u8], unpack_size: u64) -> Result<Vec<u8>, ArchiveError> {
    let mut cursor = io::Cursor::new(&reader);
    // let mut out = vec![0u8; unpack_size as usize];
    let mut out = Vec::with_capacity(unpack_size as usize);
    lzma_rs::lzma2_decompress(&mut cursor, &mut out).map_err(|e| ArchiveError::new(&format!("{:?}", e)))?;
    Ok(out)
}

/*
fn decode_lzma2(reader: &[u8], properties: &[u8], unpack_size: u64) -> Result<Vec<u8>, ArchiveError> {
    println!("Decoding with LZMA2: {:?}", reader);
    println!("Properties: {:?}", properties);
    let mut props_buffer = buffer::Buffer::new(&properties);
    let d = props_buffer.read();
    // let dictionary = read_utils::read_uint32(&mut props_buffer);

    let unpack_size = 10000;

    let lc = d % 9;
    let d  = d / 9;
    let pb = d / 5;
    let lp = d % 5;

    let props = lzma::properties::Properties {
        /// Literal context bits.
        lc,

        /// Literal position bits.
        lp,

        /// Position bits.
        pb,

        /// Dictionary size.
        dictionary: 4096,

        /// Uncompressed size if present.
        uncompressed: Some(unpack_size),
    };

    let cursor = io::Cursor::new(&reader);
    let stream = lzma::reader::Reader::new(cursor, props).map_err(|e| ArchiveError::new(&e.to_string()))?;
    Ok(read_n(stream, unpack_size))
}
*/

pub fn decode(method: &[u8], reader: &[u8], properties: &[u8], unpack_size: u64) -> Result<Vec<u8>, ArchiveError> {
    match method {
        [0x21] => decode_lzma2(reader, properties, unpack_size),
        [0x3, 0x1, 0x1] => decode_lzma(reader, properties, unpack_size),
        _ => {
            println!("Unrecognized compression ID {:?}", method);
            Err(ArchiveError::new(&format!("Unrecognized compression ID {:?}", method)))
        }
    }
}
