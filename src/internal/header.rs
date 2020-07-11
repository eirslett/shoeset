use internal::{ArchiveError};
use internal::nid::NID;
use internal::nid::read_nid;
use internal::read_utils::read_dyn_uint64 as dyn64;
use internal::buffer;
use internal::encoded_header;
use internal::read_utils;
use std::string::FromUtf16Error;
use internal::encoded_header::StreamsInfo;
use internal::encoded_header::SubstreamsInfo;

#[derive(Debug)]
pub struct Header {
    pub files_info: Vec<File>,
    pub streams_info: StreamsInfo,
    pub stream_map: StreamMap,
}

pub fn read_header(buf: &mut buffer::Buffer) -> Result<Header, ArchiveError> {
    let mut nid = read_nid(buf)?;

    let mut files_info: Vec<File> = Vec::new();
    let mut streams_info: Option<StreamsInfo> = None;

    if nid == NID::ArchiveProperties {
        skip_archive_properties(buf)?;
        nid = read_nid(buf)?;
    }

    if nid == NID::AdditionalStreamsInfo {
        return Err(ArchiveError::new("Additional streams unsupported"));
    }

    if nid == NID::MainStreamsInfo {
        streams_info = Some(encoded_header::read_streams_info(buf)?);
        nid = read_nid(buf)?;
    }

    if nid == NID::FilesInfo {
        let substreams_info = match streams_info.as_ref() {
            Some(info) => info.substreams_info.as_ref().unwrap(),
            None => return Err(ArchiveError::new("Missing substreams info"))
        };

        files_info = read_files_info(buf, &substreams_info)?;
        nid = read_nid(buf)?;
    }

    if nid != NID::End {
        return Err(ArchiveError::new(&format!("Badly terminated Header ({:?})", nid)));
    }

    let streams_info = streams_info.ok_or(ArchiveError::new("Missing StreamsInfo in the header"))?;
    let stream_map = calculate_stream_map(&files_info, &streams_info)?;
    let header = Header {
        files_info,
        stream_map,
        streams_info
    };

    Ok(header)
}

#[derive(Debug)]
pub struct Entry {
    // file: &'a File,
    pub name: String,
    pub offset: u64,
    pub size: u64,
    pub folder_index: usize,
}
pub fn get_stream_offsets(header: &Header) -> Vec<Entry> {
    let mut offsets = Vec::new();
    let mut offsets_by_folder = vec![0; header.streams_info.folders.len()];

    for i in 0..header.files_info.len() {
        let file = &header.files_info[i];
        let folder_index = header.stream_map.file_folder_index[i];
        match folder_index {
            None => { continue },
            Some(folder_index) => {
                // reopenFolderInputStream

                let uncompressed_size = file.size; // header.streams_info.pack_info.pack_sizes[first_pack_stream_index];

                let offset = offsets_by_folder[folder_index];
                // build decode stream something
                let entry = Entry {
                    // file,
                    name: file.name.to_string(),
                    offset,
                    size: uncompressed_size,
                    folder_index,
                };
                offsets.push(entry);
                offsets_by_folder[folder_index] += uncompressed_size;
            }
        };
    }
    offsets
}

#[derive(Debug)]
pub struct StreamMap {
    pub folder_first_pack_stream_index: Vec<usize>,
    pub pack_stream_offsets: Vec<usize>,
    folder_first_file_index: Vec<Option<usize>>,
    pub file_folder_index: Vec<Option<usize>>,
}
fn calculate_stream_map(files: &Vec<File>, streams_info: &StreamsInfo) -> Result<StreamMap, ArchiveError> {
    let num_folders = streams_info.folders.len();
    let mut next_folder_pack_stream_index = 0;
    let mut folder_first_pack_stream_index = Vec::with_capacity(num_folders);
    for i in 0..num_folders {
        folder_first_pack_stream_index.push(next_folder_pack_stream_index);
        next_folder_pack_stream_index += streams_info.folders[i].packed_streams.len();
    }

    let num_pack_sizes = streams_info.pack_info.pack_sizes.len();
    let mut next_pack_stream_offset: usize = 0;
    let mut pack_stream_offsets = Vec::with_capacity(num_folders);
    for i in 0..num_pack_sizes {
        pack_stream_offsets.push(next_pack_stream_offset);
        next_pack_stream_offset += streams_info.pack_info.pack_sizes[i] as usize;
    }

    let mut folder_first_file_index = vec![None; files.len()];
    let mut file_folder_index = vec![None; files.len()]; // Vec::with_capacity(files.len());
    let mut next_folder_index = 0;
    let mut next_folder_unpack_stream_index = 0;
    for i in 0..files.len() {
        if !files[i].has_stream && next_folder_unpack_stream_index == 0 {
            file_folder_index[i] = None;
            continue;
        }
        if next_folder_unpack_stream_index == 0 {
            while next_folder_index < streams_info.folders.len() {
                folder_first_file_index[next_folder_index] = Some(i);
                if streams_info.folders[next_folder_index].num_unpack_substreams > 0 {
                    break;
                }
                next_folder_index += 1;
            }
            if next_folder_index >= streams_info.folders.len() {
                return Err(ArchiveError::new("Too few folders in archive"));
            }
        }
        file_folder_index[i] = Some(next_folder_index);
        if !files[i].has_stream {
            continue;
        }

        next_folder_unpack_stream_index += 1;

        if next_folder_unpack_stream_index >= streams_info.folders[next_folder_index].num_unpack_substreams {
           next_folder_index += 1;
            next_folder_unpack_stream_index = 0;
        }
    }

    Ok(StreamMap {
        folder_first_pack_stream_index,
        pack_stream_offsets,
        folder_first_file_index,
        file_folder_index
    })
}

// Looks like these aren't really necessary
fn skip_archive_properties(buf: &mut buffer::Buffer) -> Result<(), ArchiveError> {
    let mut nid = read_nid(buf)?;
    while nid != NID::End {
        let property_size = dyn64(buf);
        buf.skip(property_size as usize);
        nid = read_nid(buf)?;
    }
    Ok(())
}

fn skip_external(buf: &mut buffer::Buffer) -> Result<(), ArchiveError> {
    let external = buf.read();
    if external != 0 {
        return Err(ArchiveError::new("External unsupported"));
    }
    Ok(())
}

fn utf16_decode(data: &[u8]) -> Result<String, FromUtf16Error> {
    // return String::from_utf8(data.to_vec());

    let mut converted: Vec<u16> = Vec::with_capacity(data.len() / 2);
    for i in 0..(data.len() / 2) {
        let first = data[2 * i] as u16;
        let second = data[2 * i + 1] as u16;
        let value = first + second * 256;
        converted.push(value);
    }
    String::from_utf16(&converted)

}

fn read_dates(buf: &mut buffer::Buffer, num_files: u64) -> Result<Vec<Option<u64>>, ArchiveError> {
    let mut dates: Vec<Option<u64>> = Vec::new();
    let times_defined = read_utils::read_all_or_bits(buf, num_files);
    skip_external(buf)?;
    for i in 0..num_files {
        dates.push(if times_defined.contains(i as usize) {
            Some(read_utils::read_uint64(buf))
        } else {
            None
        })
    }
    Ok(dates)
}

fn read_win_attributes(buf: &mut buffer::Buffer, num_files: u64) -> Result<Vec<Option<u32>>, ArchiveError> {
    let mut attrs: Vec<Option<u32>> = Vec::new();
    let times_defined = read_utils::read_all_or_bits(buf, num_files);
    skip_external(buf)?;
    for i in 0..num_files {
        attrs.push(if times_defined.contains(i as usize) {
            Some(read_utils::read_uint32(buf))
        } else {
            None
        })
    }
    Ok(attrs)
}

#[derive(Debug)]
pub struct File {
    name: String,
    has_stream: bool,
    is_directory: bool,
    is_anti_item: bool,
    creation_date: Option<u64>,
    last_modified_date: Option<u64>,
    access_date: Option<u64>,
    windows_attributes: Option<u32>,
    size: u64,
    // compressed_size: u64,
}

fn read_files_info(buf: &mut buffer::Buffer, substreams_info: &SubstreamsInfo) -> Result<Vec<File>, ArchiveError> {
    let num_files = dyn64(buf);
    let mut is_empty_stream = bit_set::BitSet::with_capacity(num_files as usize);
    let mut is_empty_file = None; // bit_set::BitSet::with_capacity(num_files as usize);
    let mut is_anti = bit_set::BitSet::with_capacity(num_files as usize);
    let mut file_names: Vec<String> = Vec::with_capacity(num_files as usize); // Vec<String> = Vec::with_capacity(num_files as usize);
    let mut file_creation_dates: Vec<Option<u64>> = vec![None; num_files as usize]; // Vec::with_capacity(num_files as usize);
    let mut file_access_dates: Vec<Option<u64>> = vec![None; num_files as usize];
    let mut file_modified_dates: Vec<Option<u64>> = vec![None; num_files as usize];
    let mut win_attributes: Vec<Option<u32>> = vec![None; num_files as usize];

    let mut files: Vec<File> = Vec::with_capacity(num_files as usize);
    loop {
        let nid = read_nid(buf)?;

        if nid == NID::End {
            break;
        }
        let size = dyn64(buf);

        match nid {
            NID::EmptyStream => is_empty_stream = read_utils::read_bits(buf, num_files),
            NID::EmptyFile => {
                is_empty_file = Some(read_utils::read_bits(buf, is_empty_stream.len() as u64));
            },
            NID::Anti => is_anti = read_utils::read_bits(buf, is_empty_stream.len() as u64),
            NID::Name => {
                skip_external(buf)?;
                if ((size - 1) & 1) != 0 {
                    return Err(ArchiveError::new("File names length invalid"));
                }


                let names = buf.read_multi((size - 1) as usize);

                let mut next_name_pos = 0;
                for x in 0..(names.len() / 2) {
                    let i = 2 * x;
                    if names[i] == 0 && names[i + 1] == 0 {
                        let name_bytes = &names[next_name_pos..i];
                        let name_str = utf16_decode(name_bytes).map_err(|e| ArchiveError::new(&e.to_string()))?;
                        file_names.push(name_str);
                        next_name_pos = i + 2;
                    }
                }
            }
            NID::Ctime => {
                file_creation_dates = read_dates(buf, num_files)?;
            }
            NID::Atime => {
                file_access_dates = read_dates(buf, num_files)?;
            }
            NID::Mtime => {
                file_modified_dates = read_dates(buf, num_files)?;
            }
            NID::WinAttributes => {
                win_attributes = read_win_attributes(buf, num_files)?;
            }
            NID::StartPos => {
                return Err(ArchiveError::new("StartPos is unsupported, please report"));
            }
            /*
            NID::Dummy => {
                buf.skip(size);
            }
            */
            _ => { buf.skip(size as usize); }
        }
    }

    let mut non_empty_file_counter = 0;
    let mut empty_file_counter = 0;

    for i in 0..(num_files as usize) {
        let has_stream = !is_empty_stream.contains(i as usize);
        if has_stream {
            files.push(File {
                name: file_names[i].to_string(),
                has_stream: true,
                is_directory: false,
                is_anti_item: false,
                creation_date: file_creation_dates[non_empty_file_counter],
                last_modified_date: file_modified_dates[non_empty_file_counter],
                access_date: file_access_dates[non_empty_file_counter],
                windows_attributes: win_attributes[non_empty_file_counter],
                size: substreams_info.unpack_sizes[non_empty_file_counter],
                // compressed_size: 0, // TODO fix
            });
            non_empty_file_counter += 1;
        } else {
            let is_directory = match &is_empty_file {
                Some(bitset) => bitset.contains(empty_file_counter),
                None => true
            };

            files.push(File {
                name: file_names[i].to_string(),
                has_stream: false,
                is_directory,
                is_anti_item: is_anti.contains(empty_file_counter),
                creation_date: None,
                last_modified_date: None,
                access_date: None,
                windows_attributes: None,
                size: 0,
                // compressed_size: 0,
            });
            empty_file_counter += 1;
        }
    }

    Ok(files)
}
