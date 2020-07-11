extern crate bit_set;
extern crate lzma_rs;
extern crate byteorder;
extern crate bytes;

#[derive(Debug, Clone)]
pub struct ArchiveError {
    pub message: String
}
impl ArchiveError {
    fn new(message: &str) -> ArchiveError {
        ArchiveError { message: String::from(message) }
    }
}

#[derive(Debug)]
pub struct InternalArchive {
    pub id: String,
    pub files: Vec<File>,
}

const SIGNATURE_HEADER_SIZE: usize = 32;
const SIGNATURE: [u8; 6] = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];

mod read_utils;
mod nid;
mod buffer;
mod header;
mod decode;
mod encoded_header;

use internal::nid::NID;
use internal::header::Header;

#[derive(Debug)]
struct StartHeader {
    next_header_offset: u64,
    next_header_size: u64,
    next_header_crc: u32
}

fn read_start_header(buf: &mut buffer::Buffer) -> Result<StartHeader, ArchiveError> {
    let next_header_offset = read_utils::read_uint64(buf);
    let next_header_size  = read_utils::read_uint64(buf);
    let next_header_crc = read_utils::read_uint32(buf);
    Ok(StartHeader {
        next_header_offset,
        next_header_size,
        next_header_crc
    })
}

pub fn decompress(data: &[u8]) -> Result<InternalArchive, ArchiveError> {
    let mut buf = buffer::Buffer::new(data);

    if data.len() < 12 {
        return Err(ArchiveError::new("The file is too small"));
    }
    for i in 0..6 {
        if data[i] != SIGNATURE[i] {
            return Err(ArchiveError::new("Signature mismatch"));
        }
    }

    let major_version = data[6];
    let minor_version = data[7];
    if major_version != 0 {
        return Err(ArchiveError::new(&format!("Unsupported 7z version ({},{})", major_version, minor_version)));
    }

    buf.seek(8);
    let _start_header_crc = 0xFFFFFFFF & read_utils::read_uint32(&mut buf);
    let start_header = read_start_header(&mut buf)?;


    buf.seek(SIGNATURE_HEADER_SIZE + start_header.next_header_offset as usize);

    let mut nid = nid::read_nid(&mut buf)?;
    if nid == NID::EncodedHeader {
        let decoded = encoded_header::read_encoded_header(&mut buf)?;
        let mut header_buf = buffer::Buffer::new(&decoded);
        nid = nid::read_nid(&mut header_buf)?;

        if nid == NID::Header {
            let header = header::read_header(&mut header_buf)?;
            return read_archive_contents(header, &mut buf);
        }
    }

    if nid == NID::Header {
        let header = header::read_header(&mut buf)?;
        return read_archive_contents(header, &mut buf);
    }

    return Err(ArchiveError::new(&format!("Unexpected NID {:?}", nid)));
}

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub data: Vec<u8>
}

fn read_archive_contents<'a>(header: Header, buf: &mut buffer::Buffer) -> Result<InternalArchive, ArchiveError> {
    let stream_offsets = header::get_stream_offsets(&header);
    println!("Stream offsets: {:?}", stream_offsets);

    let mut data: Vec<File> = Vec::new();

    let mut decoded_folders: Vec<Vec<u8>> = Vec::with_capacity(header.streams_info.folders.len());

    let num_folders = header.streams_info.folders.len();
    for folder_index in 0..num_folders {
        let folder = &header.streams_info.folders[folder_index];

        let first_pack_stream_index = header.stream_map.folder_first_pack_stream_index[folder_index];
        let folder_buf_offset =
            SIGNATURE_HEADER_SIZE as u64
                + header.streams_info.pack_info.pack_pos
                + header.stream_map.pack_stream_offsets[first_pack_stream_index] as u64;

        let compressed_size = header.streams_info.pack_info.pack_sizes[folder_index];

        buf.seek(folder_buf_offset as usize);
        let reader = buf.read_multi(compressed_size as usize);

        let coders = folder.get_ordered_coders();
        // just a little hack/shortcut; use the first coder
        let coder = &coders[0].coder_options;

        let unpack_size = folder.unpack_sizes[0]; // .iter().sum();

        let res = decode::decode(&coder.decompression_method_id, &reader, &coder.properties, unpack_size)?;

        decoded_folders.push(res);
    }

    println!("Decoded all the folders!");

    for i in 0..stream_offsets.len() {
        let entry = &stream_offsets[i];
        let mut buf = buffer::Buffer::new(&decoded_folders[entry.folder_index]);
        println!("Start = {}, Size = {}, End = {}, Available = {}", entry.offset, entry.size, entry.offset + entry.size, buf.len());
        buf.seek(entry.offset as usize);
        let result = buf.read_multi(entry.size as usize);
        data.push(File {
            name: entry.name.to_string(),
            data: result
        });
    }

    return Ok(InternalArchive{
        id: String::from("Test archive"),
        files: data
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn too_short_file_fails() {
        let bytes: [u8; 8] = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 7, 8];
        let result = decompress(&bytes);
        assert!(result.is_err(), "Should be an error");
        assert_eq!("The file is too small", result.err().expect("Should be an error").message);

    }

    #[test]
    fn wrong_signature() {
        let bytes: [u8; 14] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        let result = decompress(&bytes);
        assert!(result.is_err(), "Should be an error");
        assert_eq!("Signature mismatch", result.err().expect("Should be an error").message);
    }

    #[test]
    fn wrong_version() {
        let bytes: [u8; 14] = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C, 1, 4, 0, 0, 0, 0, 0, 0];
        let result = decompress(&bytes);
        assert!(result.is_err(), "Should be an error");
        assert_eq!("Unsupported 7z version (1,4)", result.err().expect("Should be an error").message);
    }

    #[test]
    fn it_works() -> Result<(), ArchiveError> {
        let bytes = include_bytes!("../tests/foobar.7z");
        let result = decompress(bytes)?;
        // println!("Decoded: {:?}", result);
        assert_eq!(result.files[0].name, "foobar/hello.txt");
        assert_eq!(std::str::from_utf8(&result.files[0].data).unwrap(), "catcatcatcat\n");
        assert_eq!(result.files[1].name, "foobar/world.txt");
        assert_eq!(std::str::from_utf8(&result.files[1].data).unwrap(), "dogdogdogdogdog\n");
        assert_eq!(result.id, "Test archive");
        Ok(())
    }
}
