use super::super::encoding::*;
use super::utils;

pub struct Utf16Encode {
    big_endian: bool,
}

impl EncodingStatics for Utf16Encode {
    fn new(options: &str) -> Result<Box<Encoding>, String> {
        let mut big_endian = false;
        match options {
            "" | "le" => (),
            "be" => { big_endian = true; },
            _ =>  { return Err("invalid options".into()); },
        }

        Ok(Box::new(Utf16Encode {
            big_endian: big_endian,
        }))
    }

    fn print_help() {
        println!("Encodes input character data as UTF-16.");
        println!("Options:");
        println!("  le = little endian (default)");
        println!("  be = big endian");
    }
}

impl Encoding for Utf16Encode {
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        match input.get_bytes(4) {
            Some(Ok(bytes)) => {
                let codepoint = utils::u32_from_bytes(&bytes, true);

                // Note: UTF-16's names for the "high" and "low" surrogate are somewhat confusing.
                // "High" surrogates are in the range 0xD800 - 0xDBFF.
                // "Low" surrogates are in the range 0xDC00 - 0xDFFF.
                // The names are because the "low" surrogate encodes the 10 low-order bits of the code
                // point, and the "high" surrogate encodes the 10 high-order bits.

                if codepoint >= 0xD800 && codepoint <= 0xDFFF {
                    // Forbidden range.
                    let which = if codepoint < 0xDC00 {
                        "low"
                    } else {
                        "high"
                    };
                    error!("cannot UTF-16 encode {} surrogate code point U+{:04X}", which, codepoint);
                    Some(Err(CodeError::new(format!("cannot UTF-16 encode {} surrogate code point", which))
                                       .with_bytes(bytes)))
                } else if codepoint <= 0xFFFF {
                    // Identity encoding.
                    debug!("UTF-16 trivial encoding");
                    Some(Ok(utils::u16_to_bytes(codepoint as u16, self.big_endian)))
                } else if codepoint <= 0x10_FFFF {
                    // Surrogate pair encoding (codepoint >= 0x10000)
                    debug!("UTF-16 surrogate pair encoding");
                    let subtracted = codepoint - 0x1_0000;
                    let high_surrogate = (0xD800 + (subtracted >> 10)) as u16;
                    let low_surrogate = (0xDC00 + (subtracted & 0x3FF)) as u16;
                    debug!("high = {:#X}", high_surrogate);
                    debug!("low  = {:#X}", low_surrogate);
                    let mut vec = utils::u16_to_bytes(high_surrogate, self.big_endian);
                    vec.extend_from_slice(&utils::u16_to_bytes(low_surrogate, self.big_endian));
                    Some(Ok(vec))
                } else {
                    // Codepoint > 0x10_FFFF
                    error!("cannot UTF-16 encode out-of-range code point U+{:04X}", codepoint);
                    Some(Err(CodeError::new("cannot UTF-16 encode out-of-range code point")
                                       .with_bytes(bytes)))
                }
            },
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    fn replacement(&self) -> Vec<u8> {
        utils::u16_to_bytes(::std::char::REPLACEMENT_CHARACTER as u16, self.big_endian)
    }
}

pub struct Utf16Decode {
    big_endian: bool,
}

impl EncodingStatics for Utf16Decode {
    fn new(options: &str) -> Result<Box<Encoding>, String> {
        let mut big_endian = false;
        match options {
            "" | "le" => (),
            "be" => { big_endian = true; },
            _ =>  { return Err("invalid options".into()); },
        }

        Ok(Box::new(Utf16Decode {
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

impl Utf16Decode {
    fn read_codeunit(&self, input: &mut EncodingInput, bytes: &mut Vec<u8>)
            -> Option<Result<u16, CodeError>> {
        let mut codeunit = 0u16;
        for i in 0..2 {
            match input.get_byte() {
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

impl Encoding for Utf16Decode {
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let mut bytes = vec![];

        let first_codeunit = match self.read_codeunit(input, &mut bytes) {
            Some(Ok(codeunit)) => codeunit,
            Some(Err(e)) => { return Some(Err(e)); },
            None => { return None; },
        };

        if low_surrogate(first_codeunit).is_some() {
            error!("low surrogate cannot be first in surrogate pair");
            return Some(Err(CodeError::new("low surrogate cannot be first in surrogate pair")
                                      .with_bytes(bytes)));
        }

        if let Some(value) = high_surrogate(first_codeunit) {
            debug!("high surrogate: {:04X} (value = {:08X})", first_codeunit, value);
            let mut codepoint: u32 = value + 0x1_0000;

            // Read second code unit.
            let second_codeunit = match self.read_codeunit(input, &mut bytes) {
                Some(Ok(codeunit)) => codeunit,
                Some(Err(e)) => {
                    error!("incomplete 2-unit UTF-16 codepoint: {}", e);
                    return Some(Err(CodeError::new("incomplete 2-unit UTF-16 codepoint")
                                              .with_bytes(bytes)
                                              .with_inner(e)));
                },
                None => {
                    error!("incomplete 2-unit UTF-16 codepoint due to EOF");
                    return Some(Err(CodeError::new("incomplete 2-unit UTF-16 codepoint due to EOF")
                                              .with_bytes(bytes)));
                },
            };

            if let Some(value) = low_surrogate(second_codeunit) {
                debug!("low surrogate: {:04X} (value = {:08X})", second_codeunit, value);
                codepoint |= value;
                Some(Ok(utils::u32_to_bytes(codepoint, true)))
            } else {
                error!("second code unit in surrogate pair is not a low surrogate: {:04X}", second_codeunit);
                Some(Err(CodeError::new("second code unit in surrogate pair is not a low surrogate")
                                   .with_bytes(bytes)))
            }

        } else {
            debug!("one-unit code point");
            Some(Ok(utils::u32_to_bytes(first_codeunit as u32, true)))
        }
    }

    fn replacement(&self) -> Vec<u8> {
        utils::unicode_replacement()
    }
}
