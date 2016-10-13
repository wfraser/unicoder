use super::encoding::*;

mod code_adapter;

mod hex;
pub use self::hex::*;

mod tee;
pub use self::tee::*;

mod unicode;
pub use self::unicode::*;

mod un_utf8;
pub use self::un_utf8::*;

mod utf16;
pub use self::utf16::*;

mod utf8;
pub use self::utf8::*;

mod u_code;
pub use self::u_code::*;

mod utils;

#[derive(Copy, Clone)]
pub struct CodeFunctions {
    pub new: &'static Fn(InputBox, &str) -> Result<InputBox, String>,
    pub print_help: &'static Fn(),
}

macro_rules! entry {
    ($name:expr => $typename:ident) => {
        ($name, CodeFunctions {
            new: &$typename::new,
            print_help: &$typename::print_help,
        })
    }
}

const MAP: [(&'static str, CodeFunctions); 8] = [
    entry!("hex" => HexEncode),
    entry!("tee" => Tee),
    entry!("unhex" => HexDecode),
    entry!("unicode_info" => UnicodeInfo),
    entry!("un_ucode" => UnUCode),
    entry!("un_utf8" => UnUtf8),
    entry!("utf16" => Utf16),
    entry!("utf8" => Utf8),
];

fn map_lookup(name: &str) -> Result<CodeFunctions, String> {
    let lower = name.to_lowercase();
    for pair in MAP.iter() {
        if pair.0 == &lower {
            return Ok(pair.1);
        }
    }
    Err(format!("unknown coding scheme {:?}", name))
}

pub fn get_code(name: &str, input: InputBox, options: &str) -> Result<InputBox, String> {
    match map_lookup(name) {
        Ok(functions) => (functions.new)(input, options),
        Err(e) => Err(e),
    }
}

pub fn print_help(name: &str) -> Result<(), String> {
    match map_lookup(name) {
        Ok(functions) => {
            (functions.print_help)();
            Ok(())
        },
        Err(e) => Err(e),
    }
}

pub fn print_all_help() {
    for pair in MAP.iter() {
        println!("{}:", pair.0);
        (pair.1.print_help)();
        println!("");
    }
}
