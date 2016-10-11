use super::utils;
use super::code_adapter::*;
use super::super::encoding::*;

pub struct Utf16 {
    adapter: Utf32Adapter,
}

impl CodeStatics for Utf16 {
    fn new(input: InputBox, _options: &str) -> InputBox {
        let big_endian = false;
        Box::new(Utf16 {
            adapter: Utf32Adapter::new(input, Box::new(move |cp, out| Self::process_codepoint(cp, out, big_endian))),
        })
    }

    fn print_help() {
        // TODO: add option for big-endian
        println!("(no options)");
        println!("Encodes input character data as UTF-16LE.");
    }
}

impl Utf16 {
    fn process_codepoint(codepoint: u32, out: &mut VecDequeWritable<u8>, big_endian: bool) -> Result<(), CodeError> {
        //TODO
        let mut bytes = utils::u32_to_u8_be(codepoint);

        if big_endian {
            bytes = bytes.iter().rev().map(|x| *x).collect();
        }
        for byte in bytes {
            out.push(byte);
        }
        Ok(())
    }
}

impl Code for Utf16 {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        self.adapter.next()
    }
}
