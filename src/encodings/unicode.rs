use super::code_adapter::*;
use super::super::encoding::*;

use std::io::Write;

pub struct UnicodeInfo {
    adapter: Utf32Adapter,
}

impl CodeStatics for UnicodeInfo {
    fn new(input: InputBox, _options: &str) -> InputBox {
        Box::new(UnicodeInfo {
            adapter: Utf32Adapter::new(input, Box::new(Self::process_codepoint)),
        }) as InputBox
    }

    fn print_help() {
        println!("(no options)");
    }
}

impl UnicodeInfo {
    fn process_codepoint<W: Write>(codepoint: u32, out: &mut W) -> Result<(), CodeError> {
        writeln!(out, "U+{:04X} ", codepoint).unwrap();
        // TODO: write more info about the code point. Use `ucd` crate.
        Ok(())
    }
}

impl Code for UnicodeInfo {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        self.adapter.next()
    }
}
