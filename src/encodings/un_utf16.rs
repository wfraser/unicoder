use super::super::encoding::*;

use std::collections::VecDeque;

pub struct UnUtf16 {
    input: InputBox,
    output_buffer: VecDeque<u8>,
    big_endian: bool,
}

impl CodeStatics for UnUtf16 {
    fn new(input: InputBox, options: &str) -> Result<InputBox, String> {
        let mut big_endian = false;
        match options {
            "" => (),
            "le" => (),
            "be" => { big_endian = true; },
            _ => { return Err("unutf16: invalid option".into()); }
        }

        Ok(Box::new(UnUtf16 {
            input: input,
            output_buffer: VecDeque::new(),
            big_endian: big_endian,
        }))
    }

    fn print_help() {
        println!("Decodes UTF-16 input into character data (UTF-32BE)");
        println!("Options:");
        println!("  le = little endian (UTF-16LE) input (default)");
        println!("  be = big endian (UTF-16BE) input");
    }
}

/// Returns the surrogate value shifted appropriately if it is a high surrogate, or else None.
fn high_surrogate(codeunit: u16) -> Option<u32> {
    if codeunit >= 0xD800 && codeunit <= 0xDBFF {
        Some(((codeunit - 0xD800) as u32) << 10)
    } else {
        None
    }
}

/// Returns the surrogate value shifted appropriately if it is a low surrogate, or else None.
fn low_surrogate(codeunit: u16) -> Option<u32> {
    if codeunit >= 0xDC00 && codeunit <= 0xDFFF {
        Some((codeunit - 0xDC00) as u32)
    } else {
        None
    }
}

impl UnUtf16 {
    fn push_codepoint(&mut self, codepoint: u32) {
        debug!("pushing codepoint U+{:X}", codepoint);
        self.output_buffer.push_back(((codepoint >> 24) & 0xFF) as u8);
        self.output_buffer.push_back(((codepoint >> 16) & 0xFF) as u8);
        self.output_buffer.push_back(((codepoint >> 8) & 0xFF) as u8);
        self.output_buffer.push_back((codepoint & 0xFF) as u8);
    }

    fn read_codeunit(&mut self, bytes: &mut Vec<u8>) -> Option<Result<u16, CodeError>> {
        let mut codeunit = 0u16;
        for i in 0..2 {
            match self.input.next() {
                Some(Ok(byte)) => {
                    bytes.push(byte);
                    debug!("code unit byte {}: {:02X}", i + 1, byte);
                    let shift = if self.big_endian {
                        8 * (1 - i)
                    } else {
                        8 * i
                    };
                    codeunit |= (byte as u16) << shift;
                },
                Some(Err(e)) => {
                    error!("incomplete UTF-16 code unit: {}", e);
                    return Some(Err(CodeError::new("incomplete UTF-16 code unit")
                                              .with_bytes(bytes.clone())
                                              .with_inner(e)));
                },
                None => {
                    if i == 0 {
                        return None;
                    } else {
                        error!("incomplete UTF-16 code unit due to EOF");
                        return Some(Err(CodeError::new("incomplete UTF-16 code unit due to EOF")
                                                  .with_bytes(bytes.clone())));
                    }
                }
            }
        }
        Some(Ok(codeunit))
    }

    fn parse_input(&mut self) -> Result<(), CodeError> {
        let mut bytes = vec![];

        let first_codeunit = match self.read_codeunit(&mut bytes) {
            Some(Ok(codeunit)) => codeunit,
            Some(Err(e)) => { return Err(e); },
            None => { return Ok(()); },
        };

        if low_surrogate(first_codeunit).is_some() {
            error!("low surrogate cannot be first in surrogate pair");
            return Err(CodeError::new("low surrogate cannot be first in surrogate pair")
                                 .with_bytes(bytes));
        }

        if let Some(value) = high_surrogate(first_codeunit) {
            debug!("high surrogate: {:04X} (value = {:08X})", first_codeunit, value);
            let mut codepoint: u32 = value + 0x1_0000;

            // Read second code unit.
            let second_codeunit = match self.read_codeunit(&mut bytes) {
                Some(Ok(codeunit)) => codeunit,
                Some(Err(e)) => {
                    error!("incomplete 2-unit UTF-16 codepoint: {}", e);
                    return Err(CodeError::new("incomplete 2-unit UTF-16 codepoint")
                                         .with_bytes(bytes)
                                         .with_inner(e));
                },
                None => {
                    error!("incomplete 2-unit UTF-16 codepoint due to EOF");
                    return Err(CodeError::new("incomplete 2-unit UTF-16 codepoint due to EOF")
                                         .with_bytes(bytes));
                },
            };

            if let Some(value) = low_surrogate(second_codeunit) {
                debug!("low surrogate: {:04X} (value = {:08X})", second_codeunit, value);
                codepoint |= value;
                self.push_codepoint(codepoint);
                Ok(())
            } else {
                error!("second code unit in surrogate pair is not a low surrogate: {:04X}", second_codeunit);
                Err(CodeError::new("second code unit in surrogate pair is not a low surrogate")
                              .with_bytes(bytes))
            }

        } else {
            debug!("one-unit code point");
            self.push_codepoint(first_codeunit as u32);
            Ok(())
        }
    }
}

impl Code for UnUtf16 {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        if self.output_buffer.is_empty() {
            if let Err(e) = self.parse_input() {
                return Some(Err(e));
            }
        }

        if let Some(byte) = self.output_buffer.pop_front() {
            Some(Ok(byte))
        } else {
            None
        }
    }
}
