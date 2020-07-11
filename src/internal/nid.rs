use internal::ArchiveError;
use internal::buffer;

#[derive(Debug, PartialEq)]
pub enum NID {
    End,
    Header,
    ArchiveProperties,
    AdditionalStreamsInfo,
    MainStreamsInfo,
    FilesInfo,
    PackInfo,
    UnpackInfo,
    SubStreamsInfo,
    Size,
    Crc,
    Folder,
    CodersUnpackSize,
    NumUnpackStream,
    EmptyStream,
    EmptyFile,
    Anti,
    Name,
    Ctime,
    Atime,
    Mtime,
    WinAttributes,
    Comment,
    EncodedHeader,
    StartPos,
    Dummy
}

pub fn read_nid(buf: &mut buffer::Buffer) -> Result<NID, ArchiveError> {
    let i = buf.read();
    let res = match i {
        0 => Ok(NID::End),
        1 => Ok(NID::Header),
        2 => Ok(NID::ArchiveProperties),
        3 => Ok(NID::AdditionalStreamsInfo),
        4 => Ok(NID::MainStreamsInfo),
        5 => Ok(NID::FilesInfo),
        6 => Ok(NID::PackInfo),
        7 => Ok(NID::UnpackInfo),
        8 => Ok(NID::SubStreamsInfo),
        9 => Ok(NID::Size),
        10 => Ok(NID::Crc),
        11 => Ok(NID::Folder),
        12 => Ok(NID::CodersUnpackSize),
        13 => Ok(NID::NumUnpackStream),
        14 => Ok(NID::EmptyStream),
        15 => Ok(NID::EmptyFile),
        16 => Ok(NID::Anti),
        17 => Ok(NID::Name),
        18 => Ok(NID::Ctime),
        19 => Ok(NID::Atime),
        20 => Ok(NID::Mtime),
        21 => Ok(NID::WinAttributes),
        22 => Ok(NID::Comment),
        23 => Ok(NID::EncodedHeader),
        24 => Ok(NID::StartPos),
        25 => Ok(NID::Dummy),
        _ => Err(ArchiveError::new(&format!("Unrecognized NID flag {}", i)))
    };
    println!("Read NID {:?} ({})", res, i);
    res
}

mod tests {
    #[test]
    fn read_nid_header() -> Result<(), super::ArchiveError> {
        let mut buff = super::buffer::Buffer::new(&[1]);
        let result = super::read_nid(&mut buff)?;
        assert_eq!(result, super::NID::Header);
        Ok(())
    }

    #[test]
    fn read_unexpected() {
        let mut buff = super::buffer::Buffer::new(&[250]);
        let result = super::read_nid(&mut buff);
        assert!(result.is_err(), "Should return an error");
        assert_eq!(result.err().unwrap().message, "Unrecognized NID flag 250");
    }
}
