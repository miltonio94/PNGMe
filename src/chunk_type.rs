use std::fmt;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Eq, PartialEq, Debug)]
pub struct ChunkType {
    string_value: [char; 4],
    numeric_value: [u8; 4],
    ancillary_bit: Ancillary,
    private_bit: Private,
    reserved_bit: Reserved,
    safe_to_copy_bit: SafeToCopy,
}

impl ChunkType {
    pub fn bytes(&self) -> [u8; 4] {
        self.numeric_value
    }

    pub fn is_critical(&self) -> bool {
        self.ancillary_bit == Ancillary::Critical
    }

    pub fn is_public(&self) -> bool {
        self.private_bit == Private::Private
    }

    pub fn is_reserved_bit_valid(&self) -> bool {
        self.reserved_bit == Reserved::Reserved
    }

    pub fn is_safe_to_copy(&self) -> bool {
        self.safe_to_copy_bit == SafeToCopy::SafeToCopy
    }

    pub fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }

    pub fn to_string(&self) -> String {
        self.string_value.iter().collect()
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,
               "ChunkType {{ string_value: {:?}, numeric_value: {:?}, ancillary_bit: {}, private_bit: {}, reserved_bit: {}, safe_to_copy_bit: {}}}",
            self.string_value, self.numeric_value, self.ancillary_bit, self.private_bit, self.reserved_bit, self.safe_to_copy_bit
        )
    }
}

#[derive(Eq, PartialEq, Debug)]
enum Private {
    Private,
    Public,
}

#[derive(Eq, PartialEq, Debug)]
enum Reserved {
    Reserved,
    NotReserved,
}

#[derive(Eq, PartialEq, Debug)]
enum SafeToCopy {
    SafeToCopy,
    UnsafeToCopy,
}

#[derive(Eq, PartialEq, Debug)]
enum Ancillary {
    Critical,
    Ancillary,
}

impl Display for Private {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Private::Private => write!(f, "Private"),
            Private::Public => write!(f, "Public"),
        }
    }
}

impl Display for Reserved {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Reserved::Reserved => write!(f, "Reserved"),
            Reserved::NotReserved => write!(f, "NotReserved"),
        }
    }
}

impl Display for SafeToCopy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SafeToCopy::SafeToCopy => write!(f, "SafeToCopy"),
            SafeToCopy::UnsafeToCopy => write!(f, "UnsafeToCopy"),
        }
    }
}

impl Display for Ancillary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ancillary::Ancillary => write!(f, "Ancillary"),
            Ancillary::Critical => write!(f, "Critical"),
        }
    }
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = &'static str;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        for i in value {
            if i < 65 || (i >= 91 && i <= 96) || i > 122 {
                return Result::Err(
                    r#"
Error:
Number must be a valid utf8 alpha numeric character
Remember A - Z = 65 - 90 and a - z = 97 - 122
Number outside of valid range, check PNG documentation to see valid range"#,
                );
            }
        }

        let string_value = [
            value[0] as char,
            value[1] as char,
            value[2] as char,
            value[3] as char,
        ];

        let ancillary_bit = match value[0] & (1 << 5) {
            0 => Ancillary::Critical,
            _ => Ancillary::Ancillary,
        };

        let private_bit = match value[1] & (1 << 5) {
            0 => Private::Private,
            _ => Private::Public,
        };

        let reserved_bit = match value[2] & (1 << 5) {
            0 => Reserved::Reserved,
            _ => Reserved::NotReserved,
        };

        let safe_to_copy_bit = match value[3] & (1 << 5) {
            0 => SafeToCopy::UnsafeToCopy,
            _ => SafeToCopy::SafeToCopy,
        };

        Result::Ok(ChunkType {
            string_value,
            ancillary_bit,
            private_bit,
            reserved_bit,
            safe_to_copy_bit,
            numeric_value: value,
        })
    }
}

impl FromStr for ChunkType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Result::Err("A chunk must have 4 chars");
        }
        let numeric_value = s.as_bytes();
        let numeric_value: [u8; 4] = [
            numeric_value[0],
            numeric_value[1],
            numeric_value[2],
            numeric_value[3],
        ];

        ChunkType::try_from(numeric_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _chunk_string_2 = format!("{}", chunk_type_2);
        println!("{_chunk_string}");
        println!("{_chunk_string_2}");
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
