use super::super::encoding::*;
use super::utils;

const REPLACEMENT: u8 = b'?';
const UNDEF: u32 = 0u32;

const MAPPING: [u32; 32] = [
    // 0 - 0x7F are same as Unicode
    0x20AC,  UNDEF, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, // 8
    0x02C6, 0x2030, 0x0160, 0x2039, 0x0152,  UNDEF, 0x017D,  UNDEF, // 8
     UNDEF, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014, // 9
    0x02DC, 0x2122, 0x0161, 0x203A, 0x0153,  UNDEF, 0x017E, 0x0178, // 9
    // 0xA0 - 0xFF are same as Unicode
];

pub struct Windows1252Encode;

impl EncodingStatics for Windows1252Encode {
    fn new(_options: &str) -> Result<Box<dyn Encoding>, String> {
        Ok(Box::new(Windows1252Encode))
    }

    fn print_help() {
        println!("Encodes character data as Windows-1252 (aka CP1252).");
        println!("Un-mapped characters raise a warning and are replaced with '?'.");
        println!("(no options)");
    }
}

impl Windows1252Encode {
    fn unmapped(&self, codepoint: u32) -> Option<Result<Vec<u8>, CodeError>> {
        warn!("cannot map Unicode code point U+{:04X} into Windows 1252", codepoint);
        Some(Ok(vec![REPLACEMENT]))
    }
}

impl Encoding for Windows1252Encode {
    fn next(&mut self, input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let codepoint = match input.get_bytes(4) {
            Some(Ok(read)) => {
                utils::u32_from_bytes(&read, true)
            }
            Some(Err(e)) => { return Some(Err(e)); }
            None => { return None; },
        };

        if codepoint < 0x80 || codepoint >= 0xA0 && codepoint <= 0xFF {
            debug!("U+{:04X} identity mapping", codepoint);
            return Some(Ok(vec![codepoint as u8]));
        }

        let mapped = match MAPPING.iter().enumerate().find(|&(_idx, from)| *from == codepoint) {
            Some((idx, _from)) => 0x80 + (idx as u8),
            None => { return self.unmapped(codepoint); }
        };

        debug!("U+{:04X} maps to {:#04X}", codepoint, mapped);
        Some(Ok(vec![mapped]))
    }

    fn replacement(&self) -> Vec<u8> {
        vec![REPLACEMENT]
    }
}

pub struct Windows1252Decode;

impl EncodingStatics for Windows1252Decode {
    fn new(_options: &str) -> Result<Box<dyn Encoding>, String> {
        Ok(Box::new(Windows1252Decode))
    }

    fn print_help() {
        println!("Decodes Windows 1252 (aka CP1252) into character data.");
        println!("(no options)");
    }
}

impl Encoding for Windows1252Decode {
    fn next(&mut self, input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let byte = match input.get_byte() {
            Some(Ok(byte)) => byte,
            Some(Err(e)) => { return Some(Err(e)); }
            None => { return None; }
        };

        if byte < 0x80 || byte >= 0xA0 {
            debug!("U+{:04X} identity encoding", byte);
            return Some(Ok(utils::u32_to_bytes(byte as u32, true)));
        }

        let codepoint = match MAPPING[byte as usize - 0x80] {
            UNDEF => {
                let msg = format!("Undefined Windows 1252 code unit {:#04X}", byte);
                error!("{}", msg);
                return Some(Err(CodeError::new(msg).with_bytes(vec![byte])));
            }
            codepoint => codepoint,
        };

        debug!("{:#04X} maps to U+{:04X}", byte, codepoint);
        Some(Ok(utils::u32_to_bytes(codepoint, true)))
    }

    fn replacement(&self) -> Vec<u8> {
        utils::unicode_replacement()
    }
}
