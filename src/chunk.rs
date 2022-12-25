use std::{fmt, fmt::Display};
use std::{string::FromUtf8Error, string::String};

use crate::chunk_type::ChunkType;

const DATA_TYPE_BYTES: usize = 4;
const CRC_BYTES: usize = 4;
pub const DATA_LENGTH_BYTES: usize = 4;
pub const META_DATA_BYTES: usize = DATA_TYPE_BYTES + CRC_BYTES + DATA_LENGTH_BYTES;

pub struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Chunk {{ chunk_type: {}, data: {:?} }}",
            self.chunk_type, self.data
        )
    }
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        Self { chunk_type, data }
    }

    pub fn length(&self) -> u32 {
        self.data.len() as u32
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn data_as_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.data.iter().copied().collect())
    }

    fn crc(&self) -> u32 {
        let as_bytes: Vec<u8> = self
            .chunk_type
            .bytes()
            .iter()
            .chain(self.data.iter())
            .copied()
            .collect();

        crc32fast::hash(&as_bytes)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        (self.data.len() as u32)
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type.bytes().iter())
            .chain(self.data.iter())
            .chain(self.crc().to_be_bytes().iter())
            .copied()
            .collect()
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < META_DATA_BYTES {
            return Err(ChunkError::DataSampleSmall(value.len()));
        }

        let (data_length, value) = value.split_at(DATA_LENGTH_BYTES);
        let data_length: [u8; 4] = match data_length.try_into() {
            Ok(arr) => arr,
            Err(_) => return Err(ChunkError::ParsingDataLength),
        };
        let data_length = u32::from_be_bytes(data_length) as usize;

        let (chunck_type, value) = value.split_at(DATA_TYPE_BYTES);
        let chunk_type: [u8; 4] = match chunck_type.try_into() {
            Ok(arr) => arr,
            Err(_) => return Err(ChunkError::ParsingDataType),
        };
        let chunk_type = match ChunkType::try_from(chunk_type) {
            Ok(result_chunk_type) => result_chunk_type,
            Err(chunk_type_err) => return Err(ChunkError::ParsingChunkType(chunk_type_err)),
        };

        let (data, value) = value.split_at(data_length);

        let (crc, _) = value.split_at(CRC_BYTES);
        let crc: [u8; 4] = match crc.try_into() {
            Ok(arr) => arr,
            Err(_) => return Err(ChunkError::ParsingCrc),
        };
        let crc = u32::from_be_bytes(crc);

        let chunk = Self {
            chunk_type,
            data: data.into(),
        };

        let crc_from_chunk = chunk.crc();

        if crc_from_chunk != crc {
            return Err(ChunkError::CrcNotMatching(crc, crc_from_chunk));
        }

        Ok(chunk)
    }
}

#[derive(Debug)]
pub enum ChunkError {
    DataSampleSmall(usize),
    ParsingDataLength,
    ParsingDataType,
    ParsingChunkType(&'static str),
    ParsingCrc,
    CrcNotMatching(u32, u32),
}

impl Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DataSampleSmall(length) => write!(
                f,
                "Error: size of data to small, must be at least {} bytes, data was {} bytes",
                META_DATA_BYTES, length
            ),
            Self::ParsingDataLength => write!(f, "Error: Could not parse file's meta data"),
            Self::ParsingDataType => write!(f, "Error: Could not parse file's data type"),
            Self::ParsingChunkType(chunk_type_error) => {
                write!(f, "Error: Could not parse chunk type: {}", chunk_type_error)
            }
            Self::ParsingCrc => write!(f, "Error: Could not parse CRC"),
            Self::CrcNotMatching(parsed_crc, calculated_crc) => write!(
                f,
                "Error: CRC not matching. Parsed CRC is {} and calculated CRC is {}",
                parsed_crc, calculated_crc
            ),
        }
    }
}

impl std::error::Error for ChunkError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
