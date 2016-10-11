use super::utils;
use super::code_adapter::*;
use super::super::encoding::*;

pub struct Utf8 {
    adapter: Utf32Adapter,
}

impl CodeStatics for Utf8 {
    fn new(input: InputBox, _options: &str) -> InputBox {
        Box::new(Utf8 {
            adapter: Utf32Adapter::new(input, Box::new(Self::process_codepoint)),
        }) as InputBox
    }

    fn print_help() {
        println!("(no options)");
        println!("Encodes character data (UTF-32BE) as UTF-16");
    }
}

impl Utf8 {
    fn process_codepoint(codepoint: u32, out: &mut VecDequeWritable<u8>) -> Result<(), CodeError> {
        // These ranges are illegal in Unicode, but UTF-8 can technically encode them just fine.
        if codepoint > 0x10FFFF {
            warn!("code point out of Unicode range: U+{:X}", codepoint);
        } else if codepoint >= 0xD800 && codepoint <= 0xDBFF {
            warn!("high surrogate code point U+{:X} is illegal in UTF-8", codepoint);
        } else if codepoint >= 0xDC00 && codepoint <= 0xDFFF {
            warn!("low surrogate code point U+{:X} is illegal in UTF-8", codepoint);
        }

        if codepoint < 0x80 {
            out.push(codepoint as u8);
        } else if codepoint < 0x800 {
            out.push(0b11000000 | ((codepoint >> 6) as u8));
            out.push(0b10000000 | ((codepoint & 0b00111111) as u8));
        } else if codepoint < 0x1_0000 {
            out.push(0b11100000 | ((codepoint >> (6 * 2)) as u8));
            out.push(0b10000000 | (((codepoint >> 6) & 0b00111111) as u8));
            out.push(0b10000000 | ((codepoint & 0b00111111) as u8));
        } else if codepoint < 0x20_0000 {
            out.push(0b11110000 | ((codepoint >> (6 * 3)) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 2)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> 6) & 0b00111111) as u8));
            out.push(0b10000000 | ((codepoint & 0b00111111) as u8));
        } else if codepoint < 0x400_0000 {
            out.push(0b11111000 | ((codepoint >> (6 * 4)) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 3)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 2)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> 6) & 0b00111111) as u8));
            out.push(0b10000000 | ((codepoint & 0b00111111) as u8));
        } else if codepoint < 0x8000_0000 {
            out.push(0b11111100 | ((codepoint >> (6 * 5)) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 4)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 3)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 2)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> 6) & 0b00111111) as u8));
            out.push(0b10000000 | ((codepoint & 0b00111111) as u8));
        } else {
            error!("code point out of range: cannot be represented in UTF-8: U+{:X}", codepoint);
            return Err(CodeError::new("code point out of range: cannot be represented in UTF-8")
                                 .with_bytes(utils::u32_to_u8_be(codepoint)));
        }
        Ok(())
    }
}

impl Code for Utf8 {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        self.adapter.next()
    }
}
