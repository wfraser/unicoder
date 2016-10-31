use super::super::encoding::*;
use super::Base64;
use super::utf16::{self, Utf16Encode};
use super::utils;

pub struct Utf7Encode {
    mode: Mode,
    output_buffer: Vec<u8>,
    base64: Base64,
}

fn modified_base64() -> Base64 {
    Base64 {
        code62: b'+',
        code63: b'/',
        pad: None,
    }
}

#[derive(Debug, PartialEq)]
enum Mode {
    Unicode,
    Direct,
}

impl EncodingStatics for Utf7Encode {
    fn new(_options: &str) -> Result<Box<Encoding>, String> {
        Ok(Box::new(Utf7Encode {
            mode: Mode::Direct,
            output_buffer: Vec::new(),
            base64: modified_base64(),
        }))
    }

    fn print_help() {
        println!("Encodes character data (UTF-32BE) as UTF-7");
        println!("(no options)");
    }
}

impl Utf7Encode {
    fn flush_buffer(&mut self, out: &mut Vec<u8>) {
        if !self.output_buffer.is_empty() {
            debug!("flushing buffer");
            let base64_bytes = self.base64.encode(&self.output_buffer);
            self.output_buffer.clear();
            out.extend_from_slice(&base64_bytes);
        }
    }
}

impl Encoding for Utf7Encode {
    #[allow(match_overlapping_arm, match_same_arms)]
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let mut out = vec![];
        loop {
            let bytes: Vec<u8>;
            let codepoint = match input.get_bytes(4) {
                Some(Ok(bytes_read)) => {
                    bytes = bytes_read;
                    utils::u32_from_bytes(&bytes, true)
                },
                other => { return other; }
            };

            debug!("encoding code point U+{:04X}", codepoint);

            let direct_encoding = if codepoint < 0x80 {
                match codepoint as u8 {
                    b'+' | b'\\' | b'~' => None,
                    b' ' | b'\t' | b'\r' | b'\n' | 33 ... 125 => Some(codepoint as u8),
                    _ => None,
                }
            } else {
                None
            };

            if let Some(byte) = direct_encoding {
                if self.mode != Mode::Direct {
                    debug!("switching to direct encoding");
                    self.flush_buffer(&mut out);
                    out.push(b'-');
                    self.mode = Mode::Direct;
                }
                out.push(byte);
                return Some(Ok(out));
            } else if self.mode != Mode::Unicode {
                debug!("switching to unicode encoding");
                out.push(b'+');
                self.mode = Mode::Unicode;
            }

            match Utf16Encode::encode_codepoint(codepoint, true) {
                Ok(utf16_bytes) => {
                    self.output_buffer.extend_from_slice(&utf16_bytes);
                },
                Err(e) => { return Some(Err(e)); },
            };

            if self.output_buffer.len() % 3 == 0 {
                self.flush_buffer(&mut out);
                return Some(Ok(out));
            }
        }
    }
}

pub struct Utf7Decode {
    mode: Mode,
    base64: Base64,
}

impl EncodingStatics for Utf7Decode {
    fn new(_options: &str) -> Result<Box<Encoding>, String> {
        Ok(Box::new(Utf7Decode {
            mode: Mode::Direct,
            base64: modified_base64(),
        }))
    }

    fn print_help() {
        println!("Decodes UTF-7 input into Unicode character data (UTF-32BE).");
        println!("(no options)");
    }
}

impl Encoding for Utf7Decode {
    #[allow(cyclomatic_complexity)] // yeah I know
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        loop {
            match self.mode {
                Mode::Direct => {
                    let byte = match input.get_byte() {
                        Some(Ok(byte)) => byte,
                        Some(Err(e)) => { return Some(Err(e)); },
                        None => { return None; },
                    };

                    if byte == b'+' {
                        debug!("switch to unicode mode");
                        self.mode = Mode::Unicode;
                    } else if byte > 0x7F {
                        let msg = format!("illegal {:#04X} in UTF-7 input", byte);
                        error!("{}", msg);
                        return Some(Err(CodeError::new(msg).with_bytes(vec![byte])));
                    } else {
                        debug!("direct encoding of {:?}", byte as char);
                        return Some(Ok(utils::u32_to_bytes(byte as u32, true)));
                    }
                },
                Mode::Unicode => {
                    let mut input_buffer = Vec::<u8>::with_capacity(8);
                    let mut decoded_buffer = Vec::<u16>::new();
                    let mut out = Vec::<u8>::new();

                    loop {
                        let byte = match input.get_byte() {
                            Some(Ok(byte)) => byte,
                            Some(Err(e)) => {
                                // TODO: stash the error
                                // TODO: flush buffer
                                return Some(Err(e));
                            },
                            None => {
                                break;
                            }
                        };

                        match byte {
                            b'A' ... b'Z' | b'a' ... b'z' | b'0' ... b'9' | b'+' | b'/' => {
                                // valid modified-base64 input
                                debug!("buffering {:?}", byte as char);
                                input_buffer.push(byte);
                            },
                            _ => {
                                debug!("switch to direct mode for {:?}; processing buffer",
                                        byte as char);
                                self.mode = Mode::Direct;
                                if byte != b'-' {
                                    input.unget_byte(byte);
                                }
                                break;
                            }
                        }

                        if input_buffer.len() == 8 {
                            debug!("decoding input buffer: {:?}", input_buffer);
                            // at this point we have 6 complete UTF-16 code units.
                            let utf16_bytes = match self.base64.decode(&input_buffer) {
                                Ok(bytes) => {
                                    input_buffer.clear();
                                    bytes
                                },
                                Err((_bytes, e)) => {
                                    // TODO: stash the error and process the bytes first
                                    return Some(Err(e));
                                }
                            };

                            debug!("base64-decoded into: {:?}", utf16_bytes);
                            for i in 0..3 {
                                let code_unit = utils::u16_from_bytes(&utf16_bytes[i*2 .. (i+1)*2], true);
                                decoded_buffer.push(code_unit);
                            }

                            if utf16::high_surrogate(*decoded_buffer.last().unwrap()).is_some() {
                                // Dang, we're on a surrogate pair boundary. Need to keep reading.
                                debug!("trailing surrogate pair; doing another 8-byte read");
                                continue;
                            }

                            break;
                        }
                    }

                    // Now we have some combination of decoded utf-16 and leftover undecoded bytes.
                    // Handle the rest of the bytes, discarding any partial utf-16 code units.

                    debug!("have {} base64 bytes remaining; decoding", input_buffer.len());
                    let utf16_bytes = match self.base64.decode(&input_buffer) {
                        Ok(bytes) => bytes,
                        Err((_bytes, e)) => {
                            // TODO: stash the error and process the bytes first
                            return Some(Err(e));
                        }
                    };

                    for pair in utf16_bytes.chunks(2) {
                        if pair.len() == 2 {
                            let code_unit = utils::u16_from_bytes(pair, true);
                            decoded_buffer.push(code_unit);
                        }
                    }
                    debug!("utf16 code units: {:?}", decoded_buffer);

                    // Now decode the utf-16 code units.
                    let mut partial = 0u32;
                    let mut read_high_surrogate = false;
                    for code_unit in decoded_buffer {
                        if read_high_surrogate {
                            if let Some(value) = utf16::low_surrogate(code_unit) {
                                debug!("low surrogate read");
                                partial += value + 0x1_0000;
                                debug!("U+{:04X}", partial);
                                out.extend_from_slice(&utils::u32_to_bytes(partial, true));
                                read_high_surrogate = false;
                            } else {
                                return Some(Err(CodeError::new("expected low surrogate")));
                            }
                        } else if let Some(value) = utf16::high_surrogate(code_unit) {
                            debug!("high surrogate read");
                            partial = value;
                            read_high_surrogate = true;
                        } else if utf16::low_surrogate(code_unit).is_some() {
                            return Some(Err(CodeError::new("unexpected low surrogate")));
                        } else {
                            debug!("U+{:04X}", code_unit);
                            out.extend_from_slice(&utils::u32_to_bytes(code_unit as u32, true));
                        }
                    }
                    if read_high_surrogate {
                        return Some(Err(CodeError::new("expected low surrogate")));
                    }

                    return Some(Ok(out))
                }
            }
        }
    }
}
