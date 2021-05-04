use crate::encoding::*;
use crate::encodings::utils;

const REPLACEMENT: u8 = b'?';
const SAME: u32 = 0;

const MAPPING: [u32; 256] = [
    0x0000, 0x263A, 0x263B, 0x2665, 0x2666, 0x2663, 0x2660, 0x2022, // 0
    0x25D8, 0x25CB, 0x25D9, 0x2642, 0x2640, 0x266A, 0x266B, 0x263C, // 0
    0x25BA, 0x25C4, 0x2195, 0x203C, 0x00B6, 0x00A7, 0x25AC, 0x21A8, // 1
    0x2191, 0x2193, 0x2192, 0x2190, 0x221F, 0x2194, 0x25B2, 0x25BC, // 1
    // 0x20 - 0x7E are same as Unicode
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 2
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 2
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 3
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 3
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 4
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 4
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 5
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 5
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 6
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 6
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, // 7
      SAME,   SAME,   SAME,   SAME,   SAME,   SAME,   SAME, 0x2302, // 7
    0x00C7, 0x00FC, 0x00E9, 0x00E2, 0x00E4, 0x00E0, 0x00E5, 0x00E7, // 8
    0x00EA, 0x00EB, 0x00E8, 0x00EF, 0x00EE, 0x00EC, 0x00C4, 0x00C5, // 8
    0x00C9, 0x00E6, 0x00C6, 0x00F4, 0x00F6, 0x00F2, 0x00FB, 0x00F9, // 9
    0x00FF, 0x00D6, 0x00DC, 0x00A2, 0x00A3, 0x00A5, 0x20A7, 0x0192, // 9
    0x00E1, 0x00ED, 0x00F3, 0x00FA, 0x00F1, 0x00D1, 0x00AA, 0x00BA, // A
    0x00BF, 0x2310, 0x00AC, 0x00BD, 0x00BC, 0x00A1, 0x00AB, 0x00BB, // A
    0x2591, 0x2592, 0x2593, 0x2502, 0x2524, 0x2561, 0x2562, 0x2556, // B
    0x2555, 0x2563, 0x2551, 0x2557, 0x255D, 0x255C, 0x255B, 0x2510, // B
    0x2514, 0x2534, 0x252C, 0x251C, 0x2500, 0x253C, 0x255E, 0x255F, // C
    0x255A, 0x2554, 0x2569, 0x2566, 0x2560, 0x2550, 0x256C, 0x2567, // C
    0x2568, 0x2564, 0x2565, 0x2559, 0x2558, 0x2552, 0x2553, 0x256B, // D
    0x256A, 0x2518, 0x250C, 0x2588, 0x2584, 0x258C, 0x2590, 0x2580, // D
    0x03B1, 0x00DF, 0x0393, 0x03C0, 0x03A3, 0x03C3, 0x00B5, 0x03C4, // E
    0x03A6, 0x0398, 0x03A9, 0x03B4, 0x221E, 0x03C6, 0x03B5, 0x2229, // E
    0x2261, 0x00B1, 0x2265, 0x2264, 0x2320, 0x2321, 0x00F7, 0x2248, // F
    0x00B0, 0x2219, 0x00B7, 0x221A, 0x207F, 0x00B2, 0x25A0, 0x00A0, // F
];

pub struct Cp437Encode {
    newlines: bool,
}

impl EncodingStatics for Cp437Encode {
    fn new(options: &str) -> Result<Box<dyn Encoding>, String> {
        let newlines = match options {
            "" => true,
            "nonl" => false,
            _ => return Err("unrecognized option".into()),
        };
        Ok(Box::new(Cp437Encode { newlines }))
    }

    fn print_help() {
        println!("Encodes character data as Codepage 437 (aka IBM437)");
        println!("Un-mapped characters raise a warning and are replaced with '?'.");
        println!("Caveat: many CP437 characters had multiple uses; this mapping is somewhat arbitrary.");
        println!("options:");
        println!("  nonl: encode U+000A as inverted white circle and U+000D as music note, instead of LF and CR");
    }
}

impl Encoding for Cp437Encode {
    fn next(&mut self, input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let codepoint = match input.get_bytes(4) {
            Some(Ok(read)) => {
                utils::u32_from_bytes(&read, true)
            }
            Some(Err(e)) => { return Some(Err(e)); }
            None => { return None; }
        };

        if self.newlines && (codepoint == 0x0A || codepoint == 0x0D) {
            debug!("preserving {}", if codepoint == 0x0A { "LF" } else { "CR" });
            return Some(Ok(vec![codepoint as u8]));
        }

        if MAPPING.get(codepoint as usize) == Some(&SAME) {
            debug!("U+{:04X} identity mapping", codepoint);
            return Some(Ok(vec![codepoint as u8]));
        }

        let mapped = match MAPPING.iter().enumerate().find(|&(_idx, from)| *from == codepoint) {
            Some((idx, _from)) => idx as u8,
            None => {
                warn!("cannot map Unicode code point U+{:04X} into CP437", codepoint);
                return Some(Ok(vec![REPLACEMENT]));
            }
        };

        debug!("U+{:04X} maps to {:#04X}", codepoint, mapped);
        Some(Ok(vec![mapped]))
    }

    fn replacement(&self) -> Vec<u8> {
        vec![REPLACEMENT]
    }
}

pub struct Cp437Decode {
    newlines: bool,
}

impl EncodingStatics for Cp437Decode {
    fn new(options: &str) -> Result<Box<dyn Encoding>, String> {
        let newlines = match options {
            "" => true,
            "nonl" => false,
            _ => return Err("unrecognized option".into()),
        };
        Ok(Box::new(Cp437Decode { newlines }))
    }

    fn print_help() {
        println!("Decodes Codepage 437 (aka IBM437) into character data.");
        println!("Caveat: many CP437 characters had multiple uses; this mapping is somewhat arbitrary.");
        println!("options:");
        println!("  nonl: interpret 0A as inverted white circle and 0D as music note, instead of LF and CR");
    }
}

impl Encoding for Cp437Decode {
    fn next(&mut self, input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let byte = match input.get_byte() {
            Some(Ok(byte)) => byte,
            Some(Err(e)) => return Some(Err(e)),
            None => return None,
        };

        let mut codepoint = MAPPING[byte as usize];
        if self.newlines && (byte == b'\n' || byte == b'\r') {
            debug!("preserving {}", if byte == b'\n' { "LF" } else { "CR" });
            codepoint = byte as u32;
        } else if codepoint == SAME {
            debug!("U+{:04X} identity encoding", byte);
            codepoint = byte as u32;
        } else {
            debug!("{:#04X} maps to U+{:04X}", byte, codepoint);
        }

        Some(Ok(utils::u32_to_bytes(codepoint, true)))
    }

    fn replacement(&self) -> Vec<u8> {
        utils::unicode_replacement()
    }
}
