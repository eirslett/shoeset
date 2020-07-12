use internal::{ArchiveError, SIGNATURE_HEADER_SIZE};
use internal::nid::NID;
use internal::nid::read_nid;
use internal::read_utils;
use internal::read_utils::read_dyn_uint64 as dyn64;
use internal::decode;
use std::vec::Vec;
use std::io;
use std::io::prelude::*;
use super::byteorder::ReadBytesExt;

#[derive(Debug)]
pub struct StreamsInfo {
    pub pack_info: PackInfo,
    pub substreams_info: Option<SubstreamsInfo>,
    pub folders: Vec<Folder>
}

fn or_archive_error<R>(result: Result<R, io::Error>) -> Result<R, ArchiveError> {
    result.map_err(|e| ArchiveError::new(&e.to_string()))
}

pub fn read_streams_info<R>(buf: &mut R) -> Result<StreamsInfo, ArchiveError> where R: io::BufRead {
    let mut nid = read_nid(buf)?;

    let mut pack_info: Option<PackInfo> = None;
    let mut substreams_info: Option<SubstreamsInfo> = None;

    if nid == NID::PackInfo {
        pack_info = Some(read_pack_info(buf)?);
        nid = read_nid(buf)?;
    }

    let mut folders: Vec<Folder> = Vec::new();
    if nid == NID::UnpackInfo {
        folders = read_unpack_info(buf)?;
        nid = read_nid(buf)?;
    }

    if nid == NID::SubStreamsInfo {
        substreams_info = Some(read_substreams_info(buf, &mut folders)?);
        nid = read_nid(buf)?;
    }

    if nid != NID::End {
        return Err(ArchiveError::new(&format!("Badly terminated StreamsInfo ({:?})", nid)));
    }

    Ok(StreamsInfo {
        pack_info: pack_info.unwrap(),
        substreams_info: substreams_info,
        folders
    })
}

#[derive(Debug)]
pub struct PackInfo {
    pub pack_pos: u64,
    pub pack_sizes: Vec<u64>,
    pack_crcs_defined: bit_set::BitSet,
    pack_crcs: Vec<u32>,
}
fn read_pack_info<R>(buf: &mut R) -> Result<PackInfo, ArchiveError> where R: io::BufRead {
    let pack_pos = dyn64(buf);
    let num_pack_streams = dyn64(buf);
    let mut nid = read_nid(buf)?;

    let mut pack_sizes: Vec<u64> = Vec::with_capacity(num_pack_streams as usize);
    if nid == NID::Size {
        for _ in 0..num_pack_streams {
            pack_sizes.push(dyn64(buf));
        }
        nid = read_nid(buf)?;
    }

    let mut pack_crcs_defined = bit_set::BitSet::new();
    let mut pack_crcs: Vec<u32> = Vec::new();
    if nid == NID::Crc {
        pack_crcs_defined = read_utils::read_all_or_bits(buf, num_pack_streams as usize);
        pack_crcs = (0..num_pack_streams)
            .map(|_| read_utils::read_uint32(buf))
            .collect();
        nid = read_nid(buf)?;
    }

    if nid != NID::End {
        return Err(ArchiveError::new(&format!("Badly terminated PackInfo ({:?})", nid)));
    }

    Ok(PackInfo {
        pack_pos,
        pack_sizes,
        pack_crcs_defined,
        pack_crcs
    })
}

fn read_unpack_info<R>(buf: &mut R) -> Result<Vec<Folder>, ArchiveError> where R: io::BufRead {
    let mut nid = read_nid(buf)?;
    if nid != NID::Folder {
        return Err(ArchiveError::new(&format!("Expected NID Folder, got {:?}", nid)));
    }
    let num_folders = dyn64(buf);
    let external = buf.read_u8().unwrap();
    if external != 0 {
        return Err(ArchiveError::new("External unsupported"));
    }

    let mut folders: Vec<Folder> = Vec::with_capacity(num_folders as usize);
    for _ in 0..num_folders {
        folders.push(read_folder(buf)?);
    }

    nid = read_nid(buf)?;
    if nid != NID::CodersUnpackSize {
        return Err(ArchiveError::new(&format!("Expected NID CodersUnpackSize, got {:?}", nid)));
    }

    for mut folder in folders.iter_mut() {
        folder.unpack_sizes = (0..folder.total_output_streams)
            .map(|_| dyn64(buf))
            .collect();
    }

    nid = read_nid(buf)?;

    if nid == NID::Crc {
        let crcs_defined = read_utils::read_all_or_bits(buf, num_folders as usize);
        for i in 0..num_folders {
            if crcs_defined.contains(i as usize) {
                folders[i as usize].has_crc = true;
                folders[i as usize].crc = read_utils::read_uint32(buf);
            } else {
                folders[i as usize].has_crc = false;
            }
        }
        nid = read_nid(buf)?;
    }

    if nid != NID::End {
        return Err(ArchiveError::new(&format!("Badly terminated UnpackInfo ({:?})", nid)));
    }

    Ok(folders)
}

#[derive(Debug)]
pub struct SubstreamsInfo {
    pub unpack_sizes: Vec<u64>
}
fn read_substreams_info<R>(buf: &mut R, folders: &mut Vec<Folder>) -> Result<SubstreamsInfo, ArchiveError> where R: io::BufRead {
    for folder in folders.iter_mut() {
        folder.num_unpack_substreams = 1;
    }

    let mut nid = read_nid(buf)?;
    if nid == NID::NumUnpackStream {
        for folder in folders.iter_mut() {
            folder.num_unpack_substreams = dyn64(buf);
        }
        nid = read_nid(buf)?;
    }

    let total_unpack_streams: u64 = folders.iter().map(|f| f.num_unpack_substreams).sum();

    let mut unpack_sizes: Vec<u64> = Vec::with_capacity(folders.len());
    let /* mut */ _has_crc = bit_set::BitSet::with_capacity(total_unpack_streams as usize);
    let /* mut */ _crcs: Vec<i32> = Vec::with_capacity(total_unpack_streams as usize);

    for folder in folders.iter_mut() {
        if folder.num_unpack_substreams == 0 {
            continue;
        }
        let mut sum = 0;
        if nid == NID::Size {
            for _ in 0..(folder.num_unpack_substreams - 1) {
                let size = dyn64(buf);
                unpack_sizes.push(size);
                sum += size;
            }
        }
        unpack_sizes.push(folder.get_unpack_size() - sum);
    }

    if nid == NID::Size {
        nid = read_nid(buf)?;
    }

    let mut num_digests = 0;
    for folder in folders {
        if folder.num_unpack_substreams != 1 || !folder.has_crc {
            num_digests += folder.num_unpack_substreams;
        }
    }

    if nid == NID::Crc {
        let has_missing_crc = read_utils::read_all_or_bits(buf, num_digests as usize);
        let mut missing_crcs: Vec<u32> = Vec::new();
        for i in 0..num_digests {
            if has_missing_crc.contains(i as usize) {
                missing_crcs.push(0xffffFFFF & read_utils::read_uint32(buf));
            } else {
                missing_crcs.push(0);
            }
        }
        // TODO: implement the rest of the bookkeeping here...
        /*
        let mut next_crc = 0;
        let mut next_missing_crc = 0;
        */

        nid = read_nid(buf)?;
    }

    if nid != NID::End {
        return Err(ArchiveError::new(&format!("Badly terminated SubStreamsInfo ({:?})", nid)));
    }

    Ok(SubstreamsInfo {
        unpack_sizes
    })
}

#[derive(Debug)]
pub struct Folder {
    coders: Vec<Coder>,
    bind_pairs: Vec<BindPair>,
    total_input_streams: u64,
    total_output_streams: u64,
    pub packed_streams: Vec<u64>,
    pub unpack_sizes: Vec<u64>,
    has_crc: bool,
    crc: u32,

    pub(crate) num_unpack_substreams: u64,
}

impl Folder {
    fn get_unpack_size(&self) -> u64 {
        if self.total_output_streams == 0 {
            return 0;
        }
        for i in (0..self.total_output_streams).rev() {
            if find_bind_pair_for_out_stream(&self.bind_pairs, i).is_none() {
                return self.unpack_sizes[i as usize];
            }
        }
        return 0;
    }

    pub fn get_ordered_coders(&self) -> Vec<&Coder> {
        let mut coders: Vec<&Coder> = Vec::new();
        let mut current = Some(self.packed_streams[0]);
        loop {
            match current {
                Some(curr) => {
                    coders.push(&self.coders[curr as usize]);
                    let pair = find_bind_pair_for_out_stream(&self.bind_pairs, curr);
                    current = pair.map(|p| self.bind_pairs[p].in_index);
                },
                None => {
                    return coders;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct CoderOptions {
    pub decompression_method_id: Vec<u8>,
    pub properties: Vec<u8>,
}

#[derive(Debug)]
pub struct Coder {
    pub coder_options: CoderOptions,
    num_in_streams: u64,
    num_out_streams: u64
}

#[derive(Debug)]
struct BindPair {
    in_index: u64,
    out_index: u64
}
fn find_bind_pair_for_in_stream(bind_pairs: &Vec<BindPair>, index: u64) -> Option<usize> {
    for i in 0..bind_pairs.len() {
        if bind_pairs[i].in_index == index {
            return Some(i);
        }
    }
    return None;
}
fn find_bind_pair_for_out_stream(bind_pairs: &Vec<BindPair>, index: u64) -> Option<usize> {
    for i in 0..bind_pairs.len() {
        if bind_pairs[i].out_index == index {
            return Some(i);
        }
    }
    return None;
}

fn read_folder<R>(buf: &mut R) -> Result<Folder, ArchiveError> where R: io::BufRead {
    let num_coders = dyn64(buf) as usize;

    let mut coders: Vec<Coder> = Vec::with_capacity(num_coders);
    for _ in 0..num_coders {
        let bits = buf.read_u8().unwrap();
        let id_size = bits & 0xf;
        let is_simple = (bits & 0x10) == 0;
        let has_attributes = (bits & 0x20) != 0;
        let more_alternative_methods = (bits & 0x80) != 0;

        let mut decompression_method_id = vec![0; id_size as usize];
        or_archive_error(buf.read_exact(&mut decompression_method_id))?;

        let num_in_streams = if is_simple {
            1
        } else {
            dyn64(buf)
        };

        let num_out_streams = if is_simple {
            1
        } else {
            dyn64(buf)
        };

        let mut properties: Vec<u8> = Vec::new();

        if has_attributes {
            let properties_size = dyn64(buf);
            properties = vec![0; properties_size as usize];
            or_archive_error(buf.read_exact(&mut properties))?;
            // properties = buf.read_multi(properties_size as usize);
        }

        if more_alternative_methods {
            return Err(ArchiveError::new("Alternative methods are unsupported."));
        }

        coders.push(Coder {
            coder_options: CoderOptions {
                decompression_method_id,
                properties
            },
            num_in_streams,
            num_out_streams,
        });
    }

    let total_input_streams: u64 = coders.iter().map(|c| c.num_in_streams).sum();
    let total_output_streams: u64 = coders.iter().map(|c| c.num_out_streams).sum();

    if total_output_streams == 0 {
        return Err(ArchiveError::new("Total output streams can't be 0"));
    }

    let num_bind_pairs = total_output_streams - 1;
    let mut bind_pairs = Vec::with_capacity(num_bind_pairs as usize);
    for _ in 0..num_bind_pairs {
        bind_pairs.push(BindPair {
            in_index: dyn64(buf),
            out_index: dyn64(buf)
        });
    }

    if total_input_streams < num_bind_pairs {
        return Err(ArchiveError::new("Total input streams can't be less than the number of bind pairs"));
    }

    let num_packed_streams = total_input_streams - num_bind_pairs;
    let mut packed_streams = Vec::new();
    if num_packed_streams == 1 {
        let mut idx = 0;
        for i in 0..total_input_streams {
            idx = i;
            if find_bind_pair_for_in_stream(&bind_pairs, idx).is_none() {
                break;
            }
        }
        if idx == total_input_streams {
            return Err(ArchiveError::new("Couldn't find stream's bind pair index"));
        }
        packed_streams.push(idx);
    } else {
        for _ in 0..num_packed_streams {
            packed_streams.push(dyn64(buf));
        }
    }

    Ok(Folder {
        coders,
        bind_pairs,
        total_input_streams,
        total_output_streams,
        packed_streams,
        unpack_sizes: Vec::new(),
        has_crc: false,
        crc: 0,

        num_unpack_substreams: 0,
    })
}

pub fn read_encoded_header(buf: &mut io::Cursor<&[u8]>) -> Result<Vec<u8>, ArchiveError> {
    let info = read_streams_info(buf)?;
    let folder = &info.folders[0];
    let folder_offset = SIGNATURE_HEADER_SIZE + info.pack_info.pack_pos;

    let unpack_size = folder.get_unpack_size();

    let coders = folder.get_ordered_coders();

    // just a little hack/shortcut; use the first coder
    let coder = coders[0];

    buf.set_position(folder_offset);

    let mut out = &mut vec![0u8; info.pack_info.pack_sizes[0] as usize];
    buf.read_exact(&mut out).map_err(|e| ArchiveError::new(&e.to_string()))?;
    decode::decode(&coder.coder_options.decompression_method_id, out, &coder.coder_options.properties, unpack_size)
}

mod tests {

}
