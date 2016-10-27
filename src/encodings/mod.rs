use super::encoding::*;

mod base64;
pub use self::base64::*;

mod hex;
pub use self::hex::*;

mod iso8859;
pub use self::iso8859::*;

mod normalize;
pub use self::normalize::*;

mod null;
pub use self::null::*;

mod u_code;
pub use self::u_code::*;

mod unicode_info;
pub use self::unicode_info::*;

mod utf16;
pub use self::utf16::*;

mod utf7;
pub use self::utf7::*;

mod utf8;
pub use self::utf8::*;

mod utils;

#[derive(Copy, Clone)]
pub struct CodeFunctions {
    pub new: &'static Fn(&str) -> Result<Box<Encoding>, String>,
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

const MAP: [(&'static str, CodeFunctions); 15] = [
    entry!("hex" => HexEncode),
    entry!("iso8859" => Iso8859Encode),
    entry!("normalize" => Normalize),
    entry!("null" => Null),
    entry!("ucode" => UCodeEncode),
    entry!("unicode_info" => UnicodeInfo),
    entry!("un_hex" => HexDecode),
    entry!("un_iso8859" => Iso8859Decode),
    entry!("un_ucode" => UCodeDecode),
    entry!("un_utf16" => Utf16Decode),
    entry!("un_utf7" => Utf7Decode),
    entry!("un_utf8" => Utf8Decode),
    entry!("utf16" => Utf16Encode),
    entry!("utf7" => Utf7Encode),
    entry!("utf8" => Utf8Encode),
];

fn map_lookup(name: &str) -> Result<CodeFunctions, String> {
    let lower = name.to_lowercase();
    for pair in &MAP {
        if pair.0 == &lower {
            return Ok(pair.1);
        }
    }
    Err(format!("unknown coding scheme {:?}", name))
}

pub fn get_encoding(name: &str, options: &str) -> Result<Box<Encoding>, String> {
    match map_lookup(name) {
        Ok(functions) => (functions.new)(options),
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
    for pair in &MAP {
        println!("{}:", pair.0);
        (pair.1.print_help)();
        println!("");
    }
}
