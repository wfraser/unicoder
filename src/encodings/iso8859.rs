use super::super::encoding::*;
use super::utils;

const REPLACEMENT: u8 = b'?';
const UNDEF: u32 = 0u32;

const MAPPING_NULL: [u32; 0] = [];
const MAPPING_2: [u32; 95] = [
            0x0104, 0x02d8, 0x0141, 0x00a4, 0x013d, 0x015a, 0x00a7, // A
    0x00a8, 0x0160, 0x015e, 0x0164, 0x0179, 0x00ad, 0x017d, 0x017b, // A
    0x00b0, 0x0105, 0x02db, 0x0142, 0x00b4, 0x0138, 0x015b, 0x02c7, // B
    0x00b8, 0x0161, 0x015f, 0x0165, 0x017a, 0x02dd, 0x017e, 0x017c, // B
    0x0154, 0x00c1, 0x00c2, 0x0102, 0x00c4, 0x0139, 0x0106, 0x00c7, // C
    0x010c, 0x00c9, 0x0118, 0x00cb, 0x011a, 0x00cd, 0x00ce, 0x010e, // C
    0x0110, 0x0143, 0x0147, 0x00D3, 0x00D4, 0x0150, 0x00D6, 0x00D7, // D
    0x0158, 0x016E, 0x00DA, 0x0170, 0x00DC, 0x00DD, 0x0162, 0x00DF, // D
    0x0155, 0x00E1, 0x00E2, 0x0103, 0x00E4, 0x013A, 0x0107, 0x00E7, // E
    0x010D, 0x00E9, 0x0119, 0x00EB, 0x011B, 0x00ED, 0x00EE, 0x010F, // E
    0x0111, 0x0144, 0x0148, 0x00F3, 0x00F4, 0x0151, 0x00F6, 0x00F7, // F
    0x0159, 0x016F, 0x00FA, 0x0171, 0x00FC, 0x00FD, 0x0163, 0x02D9, // F
];
const MAPPING_3: [u32; 95] = [
            0x0126, 0x02D8, 0x00A3, 0x00A4,  UNDEF, 0x0124, 0x00A7, // A
    0x00A8, 0x0130, 0x015E, 0x011E, 0x0134, 0x00AD,  UNDEF, 0x017B, // A
    0x00B0, 0x0127, 0x00B2, 0x00B3, 0x00B4, 0x00B5, 0x0125, 0x00B7, // B
    0x00B8, 0x0131, 0x015F, 0x011F, 0x0135, 0x00BD,  UNDEF, 0x017C, // B
    0x00C0, 0x00C1, 0x00C2,  UNDEF, 0x00C4, 0x010A, 0x0108, 0x00C7, // C
    0x00C8, 0x00C9, 0x00CA, 0x00CB, 0x00CC, 0x00CD, 0x00CE, 0x00CF, // C
     UNDEF, 0x00D1, 0x00D2, 0x00D3, 0x00D4, 0x0120, 0x00D6, 0x00D7, // D
    0x011C, 0x00D9, 0x00DA, 0x00DB, 0x00DC, 0x016C, 0x015C, 0x00DF, // D
    0x00E0, 0x00E1, 0x00E2,  UNDEF, 0x00E4, 0x010B, 0x0109, 0x00E7, // E
    0x00E8, 0x00E9, 0x00EA, 0x00EB, 0x00EC, 0x00ED, 0x00EE, 0x00EF, // E
     UNDEF, 0x00F1, 0x00F2, 0x00F3, 0x00F4, 0x0121, 0x00F6, 0x00F7, // F
     0x011D,0x00F9, 0x00FA, 0x00FB, 0x00FC, 0x016D, 0x015D, 0x02D9, // F
];
const MAPPING_4: [u32; 95] = [
            0x0104, 0x0138, 0x0156, 0x00A4, 0x0128, 0x013B, 0x00A7, // A
    0x00A8, 0x0160, 0x0112, 0x0122, 0x0166, 0x00AD, 0x017D, 0x00AF, // A
    0x00B0, 0x0105, 0x02DB, 0x0157, 0x00B4, 0x0129, 0x013C, 0x02C7, // B
    0x00B8, 0x0161, 0x0113, 0x0123, 0x0167, 0x014A, 0x017E, 0x014B, // B
    0x0100, 0x00C1, 0x00C2, 0x00C3, 0x00C4, 0x00C5, 0x00C6, 0x012E, // C
    0x010C, 0x00C9, 0x0118, 0x00CB, 0x0116, 0x00CD, 0x00CE, 0x012A, // C
    0x0110, 0x0145, 0x014C, 0x0136, 0x00D4, 0x00D5, 0x00D6, 0x00D7, // D
    0x00D8, 0x0172, 0x00DA, 0x00DB, 0x00DC, 0x0168, 0x016A, 0x00DF, // D
    0x0101, 0x00E1, 0x00E2, 0x00E3, 0x00E4, 0x00E5, 0x00E6, 0x012F, // E
    0x010D, 0x00E9, 0x0119, 0x00EB, 0x0117, 0x00ED, 0x00EE, 0x012B, // E
    0x0111, 0x0146, 0x014D, 0x0137, 0x00F4, 0x00F5, 0x00F6, 0x00F7, // F
    0x00F8, 0x0173, 0x00FA, 0x00FB, 0x00FC, 0x0169, 0x016B, 0x02D9, // F
];
const MAPPING_5: [u32; 95] = [
            0x0401, 0x0402, 0x0403, 0x0404, 0x0405, 0x0406, 0x0407, // A
    0x0408, 0x0409, 0x040A, 0x040B, 0x040C, 0x00AD, 0x040E, 0x040F, // A
    0x0410, 0x0411, 0x0412, 0x0413, 0x0414, 0x0415, 0x0416, 0x0417, // B
    0x0418, 0x0419, 0x041A, 0x041B, 0x041C, 0x041D, 0x041E, 0x041F, // B
    0x0420, 0x0421, 0x0422, 0x0423, 0x0424, 0x0425, 0x0426, 0x0427, // C
    0x0428, 0x0429, 0x042A, 0x042B, 0x042C, 0x042D, 0x042E, 0x042F, // C
    0x0430, 0x0431, 0x0432, 0x0433, 0x0434, 0x0435, 0x0436, 0x0437, // D
    0x0438, 0x0439, 0x043A, 0x043B, 0x043C, 0x043D, 0x043E, 0x043F, // D
    0x0440, 0x0441, 0x0442, 0x0443, 0x0444, 0x0445, 0x0446, 0x0447, // E
    0x0448, 0x0449, 0x044A, 0x044B, 0x044C, 0x044D, 0x044E, 0x044F, // E
    0x2116, 0x0451, 0x0452, 0x0453, 0x0454, 0x0455, 0x0456, 0x0457, // F
    0x0458, 0x0459, 0x045A, 0x045B, 0x045C, 0x00A7, 0x045E, 0x045F, // F
];
const MAPPING_6: [u32; 95] = [
             UNDEF,  UNDEF,  UNDEF, 0x00A4,  UNDEF,  UNDEF,  UNDEF,
     UNDEF,  UNDEF,  UNDEF,  UNDEF, 0x060C, 0x00AD,  UNDEF,  UNDEF,
     UNDEF,  UNDEF,  UNDEF,  UNDEF,  UNDEF,  UNDEF,  UNDEF,  UNDEF,
     UNDEF,  UNDEF,  UNDEF, 0x061B,  UNDEF,  UNDEF,  UNDEF, 0x061F,
     UNDEF, 0x0621, 0x0622, 0x0623, 0x0624, 0x0625, 0x0626, 0x0627,
    0x0628, 0x0629, 0x062A, 0x062B, 0x062C, 0x062D, 0x062E, 0x062F,
    0x0630, 0x0631, 0x0632, 0x0633, 0x0634, 0x0635, 0x0636, 0x0637,
    0x0638, 0x0639, 0x063A,  UNDEF,  UNDEF,  UNDEF,  UNDEF,  UNDEF,
    0x0640, 0x0641, 0x0642, 0x0643, 0x0644, 0x0645, 0x0646, 0x0647,
    0x0648, 0x0649, 0x064A, 0x064B, 0x064C, 0x064D, 0x064E, 0x064F,
    0x0650, 0x0651, 0x0652,  UNDEF,  UNDEF,  UNDEF,  UNDEF,  UNDEF,
     UNDEF,  UNDEF,  UNDEF,  UNDEF,  UNDEF,  UNDEF,  UNDEF,  UNDEF,
];

// TODO: parts 6,7,8,9,10,11,13,14,16

// Rather than writing out the whole table for this one (which would be mostly an identity
// mapping), this is just specifying the code points that are different.
const MAPPING_15: [(u8, u32); 8] = [
    (0xA4, 0x20AC), (0xA6, 0x0160), (0xA8, 0x0161), (0xB4, 0x017D),
    (0xB8, 0x017E), (0xBC, 0x0152), (0xBD, 0x0153), (0xBE, 0x0178)
];

const MAPPINGS: [&'static [u32]; 16] = [
    &MAPPING_NULL,  // 1 - Latin-1 Western European
    &MAPPING_2,     // 2 - Latin-2 Central European
    &MAPPING_3,     // 3 - Latin-3 South European
    &MAPPING_4,     // 4 - Latin-4 North European
    &MAPPING_5,     // 5 - Latin/Cyrillic
    &MAPPING_6,     // 6 - Latin/Arabic
    &MAPPING_NULL,  // 7 - Latin/Greek
    &MAPPING_NULL,  // 8 - Latin/Hebrew
    &MAPPING_NULL,  // 9 - Latin-5 Turkish
    &MAPPING_NULL,  // 10 - Latin-6 Nordic
    &MAPPING_NULL,  // 11 - Latin/Thai
    &MAPPING_NULL,  // 12 - (not used)
    &MAPPING_NULL,  // 13 - Latin-7 Baltic Rim
    &MAPPING_NULL,  // 14 - Latin-8 Celtic
    &MAPPING_NULL,  // 15 - Latin-9 (Latin-1 revision, handled specially)
    &MAPPING_NULL,  // 16 - Latin-10 South-Eastern European
];

pub struct Iso8859Encode {
    part: u8,
}

fn part_number(s: &str) -> Result<u8, String> {
    match s {
        "" => Err("no ISO 8859-N part specified".into()),
        "1" | "2" | "3" | "4" | "5" | "6" | "15" => Ok(s.parse().unwrap()),
        "7" | "8" | "9" | "10" | "11" | "13" | "14" | "16" =>
            Err(format!("ISO 8859-{} not yet implemented", s)),
        _ => Err("invalid option".into())
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

        if codepoint <= 0xA1 {
            // ASCII and C1 encoding, and identity encoding for ISO 8859-1 from Unicode.
            return Some(Ok(vec![codepoint as u8]));
        };

        let mapped = if self.part == 1 {
            if codepoint < 0x100 {
                codepoint as u8
            } else {
                return self.unmapped(codepoint);
            }
        } else if self.part == 15 {
            match MAPPING_15.iter().find(|&&(_to, from)| from == codepoint) {
                Some(&(to, _from)) => to,
                None => {
                    if codepoint < 0x100 {
                        // Part 15 is just a modification of part 1; that is, identity encoding.
                        codepoint as u8
                    } else {
                        return self.unmapped(codepoint);
                    }
                }
            }
        } else {
            let mapping = MAPPINGS[self.part as usize - 1];
            match mapping.iter().enumerate().find(|&(_, from)| *from == codepoint) {
                Some((i, _)) => 0xA1 + (i as u8),
                None => { return self.unmapped(codepoint); }
            }
        };

        debug!("U+{:04X} maps to {:#04X}", codepoint, mapped);
        Some(Ok(vec![mapped]))
    }
}

pub struct Iso8859Decode {
    part: u8,
}

impl EncodingStatics for Iso8859Decode {
    fn new(options: &str) -> Result<Box<Encoding>, String> {
        let part = try!(part_number(options));
        Ok(Box::new(Iso8859Decode {
            part: part,
        }))
    }

    fn print_help() {
        println!("Decodes ISO 8859-<N> into character data.");
        println!("Options:");
        println!("  a number 1-11 or 13-16, specifying the ISO 8859 part to use.");
    }
}

impl Encoding for Iso8859Decode {
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let byte = match input.get_byte() {
            Some(Ok(byte)) => byte,
            Some(Err(e)) => { return Some(Err(e)); },
            None => { return None; }
        };

        if byte < 0xA1 {
            return Some(Ok(utils::u32_to_bytes(byte as u32, true)));
        }

        let codepoint = if byte < 0xA1 || self.part == 1 {
            byte as u32
        } else if self.part == 15 {
            match MAPPING_15.iter().find(|&&(from, _to)| from == byte) {
                Some(&(_from, to)) => to,
                None => byte as u32
            }
        } else {
            let mapping = MAPPINGS[self.part as usize - 1];
            match mapping[byte as usize - 0xA1] {
                UNDEF => {
                    let msg = format!("undefined ISO 8859-{} code unit {:#04X}", self.part, byte);
                    error!("{}", msg);
                    return Some(Err(CodeError::new(msg).with_bytes(vec![byte])));
                },
                codepoint => codepoint
            }
        };

        debug!("{:#04X} maps to U+{:04X}", byte, codepoint);
        Some(Ok(utils::u32_to_bytes(codepoint, true)))
    }

    fn replacement(&self) -> Vec<u8> {
        utils::unicode_replacement()
    }
}
