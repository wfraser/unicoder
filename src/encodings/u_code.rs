use super::super::encoding::*;
use super::utils;

use std::char;
use std::io::Write;

pub struct UCodeDecode;

impl EncodingStatics for UCodeDecode {
    fn new(_options: &str) -> Result<Box<Encoding>, String> {
        Ok(Box::new(UCodeDecode))
    }

    fn print_help() {
        println!("Parses whitespace-separated 'U+XXXX' sequences into character data (UTF-32BE)");
        println!("(no options)");
    }
}

fn hex_digit_value(c: u8) -> Option<u8> {
    if c >= b'0' && c <= b'9' {
        Some(c - b'0')
    } else if c >= b'a' && c <= b'f' {
        Some(c - b'a' + 10)
    } else if c >= b'A' && c <= b'F' {
        Some(c - b'A' + 10)
    } else {
        None
    }
}

fn unexpected(bytes: Vec<u8>, expected: Option<&'static str>) -> Option<Result<Vec<u8>, CodeError>> {
    let mut msg = format!("unexpected {:?}", *bytes.last().unwrap() as char);
    if let Some(expected) = expected {
        msg.push_str(&format!(", execting {}", expected));
    }
    msg.push_str(" while parsing U+ code");
    error!("{}", msg);
    Some(Err(CodeError::new(msg).with_bytes(bytes)))
}

fn error(msg: &'static str, bytes: Vec<u8>, error: Option<CodeError>) -> Option<Result<Vec<u8>, CodeError>> {
    if let Some(error) = error {
        error!("{} while reading U+ code: {}", msg, error);
        Some(Err(CodeError::new("Error reading U+ code").with_bytes(bytes).with_inner(error)))
    } else {
        error!("{} while reading U+ code", msg);
        Some(Err(CodeError::new("EOF while reading U+ code").with_bytes(bytes)))
    }
}

impl Encoding for UCodeDecode {
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let mut codepoint = 0u32;
        let mut bytes = vec![];

        // Find a character 'U', ignoring whitespace.
        loop {
            match input.get_byte() {
                Some(Ok(byte)) => {
                    bytes.push(byte);

                    match byte {
                        b'U' => { break; },
                        b' ' | b'\t' | b'\r' | b'\n' => { continue; },
                        _ => { return unexpected(bytes, Some("whitespace or 'U'")); }
                    }
                },
                Some(Err(e)) => { return Some(Err(e)); },
                None => { return None; },
            }
        }

        // Read a '+'.
        match input.get_byte() {
            Some(Ok(byte)) => {
                if byte != b'+' {
                    return unexpected(bytes, Some("'+'"));
                }
            },
            Some(Err(e)) => { return error("error", bytes, Some(e)); },
            None => { return error("EOF", bytes, None); }
        }

        // Read a minimum of 4 hex digits.
        match input.get_bytes(4) {
            Some(Ok(read)) => {
                bytes.extend(&read);
                for (i, byte) in read.iter().enumerate() {
                    let value = match hex_digit_value(*byte) {
                        Some(v) => v,
                        None => {
                            return error("got garbage while expecting hex digit", bytes, None);
                        }
                    };

                    codepoint |= (value as u32) << (4 * (3 - i));
                }
            },
            Some(Err(e)) => { return error("error", bytes, Some(e)); }
            None => { return error("EOF", bytes, None); }
        }

        // Read up to two more hex digits.
        for _ in 4..6 {
            match input.get_byte() {
                Some(Ok(byte)) => {
                    bytes.push(byte);
                    let value = match hex_digit_value(byte) {
                        Some(v) => v,
                        None => {
                            // Not a hex digit; we're done with this codepoint.
                            break;
                        },
                    };

                    codepoint <<= 4;
                    codepoint |= value as u32;
                },
                Some(Err(e)) => { return error("error", bytes, Some(e)); },
                None => { break; }
            }
        }

        Some(Ok(utils::u32_to_bytes(codepoint, true)))
    }
}

pub struct UCodeEncode;

impl EncodingStatics for UCodeEncode {
    fn new(_options: &str) -> Result<Box<Encoding>, String> {
        Ok(Box::new(UCodeEncode))
    }

    fn print_help() {
        println!("Formats character data (UTF-32BE) as U+XXXX codes, separated by spaces.");
        println!("(no options)");
    }
}

impl Encoding for UCodeEncode {
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        match input.get_bytes(4) {
            Some(Ok(bytes)) => {
                let codepoint = utils::u32_from_bytes(&bytes, true);
                let mut out = vec![];
                write!(out, "U+{:04X} ", codepoint).unwrap();
                Some(Ok(out))
            },
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}
