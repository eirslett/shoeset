#[derive(Debug, Clone)]
pub struct ArchiveError {
    pub message: String
}
impl ArchiveError {
    fn new(message: &str) -> ArchiveError {
        ArchiveError { message: String::from(message) }
    }
}

pub struct InternalArchive {
    pub id: String
}

const SIGNATURE: [u8; 6] = [0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];

pub fn decompress(data: &[u8]) -> Result<InternalArchive, ArchiveError> {
    // log("decompress here from Rust");
    // log(&format!("Length is {}", data.len()));

    for i in 0..6 {
        if data[i] != SIGNATURE[i] {
            println!("Mismatch");
            return Err(ArchiveError::new("Signature mismatch"))
        } else {
            println!("OK");
        }
    }

    return Ok(InternalArchive{
        id: String::from("Test archive")
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let bytes = include_bytes!("../tests/foobar.7z");
        let result = decompress(bytes);
        match result {
            Ok(res) => { assert_eq!(res.id, "Test archive"); }
            Err(e) => { panic!(e); }
        }
    }
}
