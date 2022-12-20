use clap::{arg, builder::PossibleValue, command, value_parser, Command, ValueEnum};
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;

use crate::chunk_type;

//EXAMPLES of commands for this program
//
// pngme encode ./dice.png ruSt "This is a secret message!
//
// pngme decode ./dice.png ruSt
//
// pngme remove ./dice.png ruSt
//
// pngme print ./dice.png

pub struct Arguments {
    action: Action,
    file_path: PathBuf,
    chunk_type: Option<String>,
    message: Option<String>,
    output_path: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Action {
    Encode,
    Decode,
    Remove,
    Print,
}

impl FromStr for Action {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, false) {
                return Ok(*variant);
            }
        }
        Err(format!("Invalid variant: {}", s))
    }
}

// impl ValueEnum for Action {
//     fn value_variants<'a>() -> &'a [Self] {
//         &[Action::Encode, Action::Decode, Action::Remove, Action::Print]
//     }

//     fn to_possible_value<'a>(&self) -> Option<PossibleValue> {
//         Some(match self {
//             Self::Decode => PossibleValue::new("decode").help("decode msg from png"),
//             Self::Encode => PossibleValue::new("decode").help("encode msg onto png"),
//             Self::Remove => PossibleValue::new("decode").help("remove msg from png"),
//             Self::Print => PossibleValue::new("decode").help("print msg from png"),
//         })
//     }
// }

impl Arguments {
    pub fn parse_arguments<'a>() -> Arguments {
        let matches = Command::new("PngMe")
            .version("0.1")
            .author("Milton")
            .about("Does things involving pngs")
            .arg(
                arg!(<ACTION>)
                    .help("What action do you want to perform")
                    .value_parser(value_parser!(Action))
                    .required(true),
            )
            .arg(
                arg!(<FILE_PATH>)
                    .help("Path to the file you want to operate on")
                    .value_parser(value_parser!(String))
                    .required(true),
            )
            .arg(
                arg!(<CHUNK_TYPE>)
                    .help("Which chunk type would you like to target")
                    .value_parser(value_parser!(String))
                    .required(false),
            )
            .arg(
                arg!(<MESSAGE>)
                    .help("What message would you like to embed in the file")
                    .value_parser(value_parser!(String))
                    .required(false),
            )
            .arg(
                arg!(<OUTPUT_PATH>)
                    .help("Optional: Path for output")
                    .value_parser(value_parser!(String))
                    .required(false),
            )
            .get_matches();

        let action = *matches
            .get_one::<Action>("ACTION")
            .expect("<ACTION> is required");

        let file_path = matches
            .get_one::<String>("FILE_PATH")
            .expect("<FILE_PATH> is required");
        let file_path = PathBuf::from(file_path);

        let chunk_type = match matches.get_one::<String>("CHUNK_TYPE") {
            Some(string) => Some(string.chars().collect()),
            None => None,
        };

        let message = match matches.get_one::<String>("MESSAGE") {
            Some(string) => Some(string.chars().collect()),
            None => None,
        };

        let output_path = match matches.get_one::<String>("OUTPUT_PATH") {
            Some(path) => Some(PathBuf::from(path)),
            None => None,
        };

        let arguments = Arguments {
            action,
            file_path,
            chunk_type,
            message,
            output_path,
        };

        arguments
    }

    pub fn action_has_enough_data(arguments: &Arguments) -> Result<(), ArgsErr> {
        if arguments.action == Action::Encode
            && arguments.chunk_type.is_none()
            && arguments.message.is_none()
        {
            return Err(ArgsErr::MissingMessageAndChunkType);
        }
        if arguments.action == Action::Decode && arguments.chunk_type.is_none() {
            return Err(ArgsErr::MissingChunkType);
        }
        if arguments.action == Action::Remove && arguments.chunk_type.is_none() {
            return Err(ArgsErr::MissingChunkType);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ArgsErr {
    MissingChunkType,
    MissingMessageAndChunkType,
}

impl std::error::Error for ArgsErr {}

impl Display for ArgsErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingChunkType => write!(
                f,
                "Missing Chunk Type from your argument list, use -h flag to learn how to use"
            ),
            Self::MissingMessageAndChunkType => write!(
                f,
                "Missing Chunk Type and Message from your argument list, use -h flag to learn how to use"
            )
        }
    }
}
