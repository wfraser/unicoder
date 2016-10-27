use super::super::encoding::*;
use super::utils;

use std::error::Error;

pub struct Utf8Encode;

impl EncodingStatics for Utf8Encode {
    fn new(_options: &str) -> Result<Box<Encoding>, String> {
        Ok(Box::new(Utf8Encode))
    }

    fn print_help() {
        println!("Encodes character data (UTF-32BE) as UTF-16");
        println!("(no options)");
    }
}

impl Encoding for Utf8Encode {
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let bytes: Vec<u8>;
        let codepoint = match input.get_bytes(4) {
            Some(Ok(bytes_read)) => {
                bytes = bytes_read;
                utils::u32_from_bytes(&bytes, true)
            },
            other => { return other; }
        };

        debug!("endcoding code point U+{:04X}", codepoint);

        // These ranges are illegal in Unicode, but UTF-8 can technically encode them just fine.
        if codepoint > 0x10FFFF {
            warn!("code point out of Unicode range: U+{:X}", codepoint);
        } else if codepoint >= 0xD800 && codepoint <= 0xDBFF {
            warn!("high surrogate code point U+{:X} is illegal in UTF-8", codepoint);
        } else if codepoint >= 0xDC00 && codepoint <= 0xDFFF {
            warn!("low surrogate code point U+{:X} is illegal in UTF-8", codepoint);
        }

        let mut out = vec![];
        if codepoint < 0x80 {
            debug!("1-byte codepoint");
            out.push(codepoint as u8);
        } else if codepoint < 0x800 {
            debug!("2-byte codepoint");
            out.push(0b11000000 | ((codepoint >> 6) as u8));
            out.push(0b10000000 | ((codepoint & 0b00111111) as u8));
        } else if codepoint < 0x1_0000 {
            debug!("3-byte codepoint");
            out.push(0b11100000 | ((codepoint >> (6 * 2)) as u8));
            out.push(0b10000000 | (((codepoint >> 6) & 0b00111111) as u8));
            out.push(0b10000000 | ((codepoint & 0b00111111) as u8));
        } else if codepoint < 0x20_0000 {
            debug!("4-byte codepoint");
            out.push(0b11110000 | ((codepoint >> (6 * 3)) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 2)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> 6) & 0b00111111) as u8));
            out.push(0b10000000 | ((codepoint & 0b00111111) as u8));
        } else if codepoint < 0x400_0000 {
            debug!("5-byte codepoint");
            out.push(0b11111000 | ((codepoint >> (6 * 4)) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 3)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 2)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> 6) & 0b00111111) as u8));
            out.push(0b10000000 | ((codepoint & 0b00111111) as u8));
        } else if codepoint < 0x8000_0000 {
            debug!("6-byte codepoint");
            out.push(0b11111100 | ((codepoint >> (6 * 5)) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 4)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 3)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> (6 * 2)) & 0b00111111) as u8));
            out.push(0b10000000 | (((codepoint >> 6) & 0b00111111) as u8));
            out.push(0b10000000 | ((codepoint & 0b00111111) as u8));
        } else {
            error!("code point out of range: cannot be represented in UTF-8: U+{:X}", codepoint);
            return Some(Err(CodeError::new("code point out of range: cannot be represented in UTF-8")
                                      .with_bytes(bytes)));
        }
        Some(Ok(out))
    }

    fn replacement(&self) -> Vec<u8> {
        vec![0xEF, 0xBF, 0xBD]  // U+FFFD (::std::char::REPLACEMENT_CHARACTER) in UTF-8
    }
}

pub struct Utf8Decode;

impl EncodingStatics for Utf8Decode {
    fn new(_options: &str) -> Result<Box<Encoding>, String> {
        Ok(Box::new(Utf8Decode))
    }

    fn print_help() {
        // TODO: add a "strict" mode that checks for overlong sequences (or maybe invert that and
        // have a "relaxed" mode)
        // TODO: add a mode that yields substitution characters instead of errors
        println!("Decodes UTF-8 input into character data (UTF-32BE)");
        println!("(no options)");
    }
}

fn incomplete_error(nbytes: u8, bytes: Vec<u8>, error: Option<Box<Error>>)
        -> Option<Result<Vec<u8>, CodeError>> {
    let last_byte = *bytes.last().unwrap();
    let mut msg = format!("incomplete multi-byte code point: expected {} bytes, only got {}", nbytes, bytes.len() - 1);
    if let Some(ref e) = error {
        msg.push_str(&format!(" due to error: {}", e));
    } else if last_byte < 0b10000000 {
        msg.push_str(&format!(" due to single-byte codepoint {:#x}", last_byte));
    } else if last_byte >= 0b11100000 {
        msg.push_str(&format!(" due to unexpected initial byte {:#x}", last_byte));
    } else {
        msg.push_str(" due to EOF");
    }

    let mut code_error = CodeError::new(msg).with_bytes(bytes);
    code_error.set_inner(error);
    Some(Err(code_error))
}

impl Encoding for Utf8Decode {
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let mut bytes = vec![];

        let first_byte = match input.get_byte() {
            Some(Ok(byte)) => byte,
            Some(Err(e)) => { return Some(Err(e)); },
            None => { return None },
        };
        bytes.push(first_byte);

        let mut codepoint: u32;
        let nbytes: u8;
        if first_byte < 0b10000000 {
            debug!("{:#x}: single byte sequence", first_byte);
            codepoint = first_byte as u32;
            nbytes = 1;
            //return Some(Ok(utils::u32_to_bytes(first_byte as u32, true)));
        } else if first_byte < 0b11100000 {
            // 2 bytes:
            // 110ABCDE 10FGHIJK
            codepoint = ((first_byte & 0b00011111) as u32) << 6;
            nbytes = 2;
        } else if first_byte < 0b11110000 {
            // 3 bytes:
            // 1110ABCD 10EFGHIJ 10KLMNOP
            codepoint = ((first_byte & 0b00001111) as u32) << (6 * 2);
            nbytes = 3;
        } else if first_byte < 0b11111000 {
            // 4 bytes:
            // 11110ABC 10DEFGHI 10JKLMNO 10PQRSTU
            codepoint = ((first_byte & 0b00000111) as u32) << (6 * 3);
            nbytes = 4;
        } else if first_byte < 0b11111100 {
            // 5 bytes:
            // 111110AB 10CDEFGH 10IJKLMN 10OPQRST 10UVWXYZ
            codepoint = ((first_byte & 0b00000011) as u32) << (6 * 4);
            nbytes = 5;
        } else if first_byte < 0b11111110 {
            // 6 bytes:
            // 1111110A 10BCDEFG 10HIJKLM 10NOPQRS 10TUVWXY 10ZABCDE
            codepoint = ((first_byte & 0b00000001) as u32) << (6 * 5);
            nbytes = 6;
        } else {
            // byte == 0b11111111 or 0b11111110
            error!("illegal byte {:#x}", first_byte);
            return Some(Err(CodeError::new("illegal byte")
                                      .with_bytes(bytes)));
        }

        if nbytes > 1 {
            debug!("{:#x}: initial byte of {}-byte sequence", first_byte, nbytes);
        }
        debug!("{:032b}, {} bytes, shift = {}, buffer = {:?}", codepoint, nbytes, 6 * (nbytes - 1), &bytes);

        for i in 1..nbytes {
            let byte = match input.get_byte() {
                Some(Ok(byte)) => byte,
                Some(Err(e)) => { return incomplete_error(nbytes, bytes, Some(Box::new(e))); },
                None => { return incomplete_error(nbytes, bytes, None); },
            };
            bytes.push(byte);

            if byte < 0b10000000 || byte >= 0b11000000 {
                // unexpected single-byte or initial byte
                input.unget_byte(byte);
                return incomplete_error(nbytes, bytes, None);
            } else {
                // continuation byte
                let shift = 6 * (nbytes - i - 1);
                codepoint |= ((byte & 0b00111111) as u32) << shift;
                debug!("{:#x}: continuation byte", byte);
                debug!("{:032b}, {} bytes, shift = {}, buffer = {:?}", codepoint, nbytes, shift, bytes);
            }
        }

        debug!("got U+{:04X}", codepoint);

        if (nbytes == 2 && codepoint < 0x80)
                || (nbytes == 3 && codepoint < 0x800)
                || (nbytes == 4 && codepoint < 0x1_0000)
                || nbytes > 4 {
            warn!("overlong sequence: {:?}", bytes);
            // TODO: raise error here if in strict mode
        }

        Some(Ok(utils::u32_to_bytes(codepoint, true)))
    }

    fn replacement(&self) -> Vec<u8> {
        utils::unicode_replacement()
    }
}
