use super::encoding::*;

mod hex;
pub use self::hex::*;

mod unicode;
pub use self::unicode::*;

mod unutf8;
pub use self::unutf8::*;

#[derive(Copy, Clone)]
pub struct CodeFunctions {
    pub new: &'static Fn(InputBox, &str) -> InputBox,
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

const MAP: [(&'static str, CodeFunctions); 4] = [
    entry!("hex" => HexEncode),
    entry!("unhex" => HexDecode),
    entry!("unicode_info" => UnicodeInfo),
    entry!("unutf8" => UnUtf8),
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
        Ok(functions) => Ok((functions.new)(input, options)),
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
