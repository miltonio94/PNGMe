use std::{fmt, fmt::Display};
use std::{string::String, string::FromUtf8Error};

use crate::chunk_type::ChunkType;

const DATA_TYPE_BYTES: usize = 4;
const CRC_BYTES: usize = 4;
const DATA_LENGTH_BYTES: usize = 4;
const META_DATA_BYTES: usize = DATA_TYPE_BYTES + CRC_BYTES + DATA_LENGTH_BYTES;

struct Chunk {
    chunk_type: ChunkType,
    data: Vec<u8>,
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Chunk {{ chunk_type: {}, data: {:?} }}", self.chunk_type, self.data)
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
    type Error = &'static str;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < META_DATA_BYTES {
            return Result::Err(
                "ERROR: Length of input too small, must be at least 12 bytes in length",
            );
        }

        let (data_length, value) = value.split_at(DATA_LENGTH_BYTES);
        let data_length: [u8; 4] = match data_length.try_into() {
            Ok(arr) => arr,
            Err(_) => return  Result::Err("ERROR: Something went wrong in the parsing of the meta data, data length bytes, please contact author of program")
        };
        let data_length = u32::from_be_bytes(data_length) as usize;

        let (chunck_type, value) = value.split_at(DATA_TYPE_BYTES);
        let chunk_type: [u8; 4] = match chunck_type.try_into() {
            Ok(arr) => arr,
            Err(_) => return  Result::Err("ERROR: Something went wrong in the parsing of the meta data, data type bytes, please contact author of program")
        };
        let chunk_type = ChunkType::try_from(chunk_type)?;

        if !chunk_type.is_valid() {
            return Err("ERROR: Invalid Chunk type");
        }

        let (data, value) = value.split_at(data_length);

        let (crc, _) = value.split_at(CRC_BYTES);
        let crc: [u8; 4] = match crc.try_into() {
            Ok(arr) => arr,
            Err(_) => return Result::Err("ERROR parsing CRC from stream of data"),
        };
        let crc = u32::from_be_bytes(crc);

        let chunk = Self {
            chunk_type,
            data: data.into(),
        };

        let crc_from_chunk = chunk.crc();

        if crc_from_chunk != crc {
            return Result::Err("ERROR: Calculated CRC and CRC from stream of data do not match");
        }

        Ok(chunk)
    }
}

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
        // TODO: NOT PASSING
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        // TODO: NOT PASSING
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        // TODO: NOT PASSING
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        // TODO: NOT PASSING
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        // TODO: NOT PASSING
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
        // TODO: NOT PASSING
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
