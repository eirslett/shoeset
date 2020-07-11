use internal::ArchiveError;
use std::io;
use std::io::Read;

fn decode_lzma(reader: &[u8], properties: &[u8], unpack_size: u64) -> Result<Vec<u8>, ArchiveError> {
    let mut cursor = properties.chain(reader);
    let mut out = Vec::with_capacity(unpack_size as usize); // vec![0u8; unpack_size as usize];

    let unpacked_size = lzma_rs::decompress::UnpackedSize::UseProvided(Some(unpack_size));
    lzma_rs::lzma_decompress_with_options(&mut cursor, &mut out, &lzma_rs::decompress::Options { unpacked_size }).map_err(|e| ArchiveError::new(&format!("{:?}", e)))?;

    Ok(out)

}

fn decode_lzma2(reader: &[u8], _properties: &[u8], unpack_size: u64) -> Result<Vec<u8>, ArchiveError> {
    let mut cursor = io::Cursor::new(&reader);
    let mut out = Vec::with_capacity(unpack_size as usize);
    lzma_rs::lzma2_decompress(&mut cursor, &mut out).map_err(|e| ArchiveError::new(&format!("{:?}", e)))?;
    Ok(out)
}

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
