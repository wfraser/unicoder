use super::super::encoding::*;
use super::utils;

const REPLACEMENT: u8 = b'?';

pub struct Iso8859Encode {
    part: u8,
}

fn part_number(s: &str) -> Result<u8, String> {
    match s {
        "" => Err(format!("no ISO 8859-N part specified")),
        "1" => Ok(1),
        "15" => Ok(15),
        "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11" | "13" | "14" | "16" =>
            Err(format!("ISO 8859-{} not yet implemented", s)),
        _ => Err(format!("invalid option"))
    }
}

impl EncodingStatics for Iso8859Encode {
    fn new(options: &str) -> Result<Box<Encoding>, String> {
        let part = try!(part_number(options));
        Ok(Box::new(Iso8859Encode {
            part: part,
        }))
    }

    fn print_help() {
        println!("Encodes character data as ISO 8859-<N>. Un-mapped characters raise a warning,");
        println!("  and are replaced with '?'.");
        println!("Options:");
        println!("  a number 1-11 or 13-16, specifying the ISO 8859 part to use.");
    }
}

impl Iso8859Encode {
    fn unmapped(&self, codepoint: u32) -> Option<Result<Vec<u8>, CodeError>> {
        warn!("cannot map Unicode code point U+{:04X} into ISO 8859-{}",
              codepoint, self.part);
        Some(Ok(vec![REPLACEMENT]))
    }
}

impl Encoding for Iso8859Encode {
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let bytes: Vec<u8>;
        let codepoint = match input.get_bytes(4) {
            Some(Ok(read)) => {
                bytes = read;
                utils::u32_from_bytes(&bytes, true)
            },
            Some(Err(e)) => { return Some(Err(e)); },
            None => { return None; },
        };

        if codepoint < 0xA0 {
            // ASCII and C1 encoding, and identity encoding for ISO 8859-1 from Unicode.
            return Some(Ok(vec![codepoint as u8]));
        };

        let mapped: u8 = match self.part {
            1 => {
                if codepoint < 0x100 {
                    codepoint as u8
                } else {
                    return self.unmapped(codepoint);
                }
            },
            15 => {
                match codepoint {
                    0x20AC => 0xA4,
                    0x0160 => 0xA6,
                    0x0161 => 0xA8,
                    0x017D => 0xB4,
                    0x017E => 0xB8,
                    0x0152 => 0xBC,
                    0x0153 => 0xBD,
                    0x0178 => 0xBE,
                    other if other < 0x100 => (other as u8),
                    _ => { return self.unmapped(codepoint); }
                }
            },
            _ => unreachable!()
        };
        debug!("U+{:04X} maps to {:02X}", codepoint, mapped);
        Some(Ok(vec![mapped]))
    }
}

pub struct Iso8859Decode {
    _part: u8,
}

impl EncodingStatics for Iso8859Decode {
    fn new(options: &str) -> Result<Box<Encoding>, String> {
        let part = try!(part_number(options));
        Ok(Box::new(Iso8859Decode {
            _part: part,
        }))
    }

    fn print_help() {
        println!("Decodes ISO 8859-<N> into character data.");
        println!("Options:");
        println!("  a number 1-11 or 13-16, specifying the ISO 8859 part to use.");
    }
}

impl Encoding for Iso8859Decode {
    fn next(&mut self, _input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        Some(Err(CodeError::new("ISO 8859 decode not implemented yet.")))
    }
}
