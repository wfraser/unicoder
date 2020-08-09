use super::super::encoding::*;
use super::utils;

//use encoding_index_japanese;

const HALFWIDTH: [char; 63] = [
        '｡','｢','｣','､','･','ｦ','ｧ','ｨ','ｩ','ｪ','ｫ','ｬ','ｭ','ｮ','ｯ',
    'ｰ','ｱ','ｲ','ｳ','ｴ','ｵ','ｶ','ｷ','ｸ','ｹ','ｺ','ｻ','ｼ','ｽ','ｾ','ｿ',
    'ﾀ','ﾁ','ﾂ','ﾃ','ﾄ','ﾅ','ﾆ','ﾇ','ﾈ','ﾉ','ﾊ','ﾋ','ﾌ','ﾍ','ﾎ','ﾏ',
    'ﾐ','ﾑ','ﾒ','ﾓ','ﾔ','ﾕ','ﾖ','ﾗ','ﾘ','ﾙ','ﾚ','ﾛ','ﾜ','ﾝ','ﾞ','ﾟ'
];

pub struct ShiftJISEncode;

impl EncodingStatics for ShiftJISEncode {
    fn new(_options: &str) -> Result<Box<dyn Encoding>, String> {
        Err("Shift JIS encoding is not implemented yet.".to_owned())
    }

    fn print_help() {
        println!("Encodes character data as Shift JIS (not implemented yet)");
    }
}

impl Encoding for ShiftJISEncode {
    fn next(&mut self, _input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        unimplemented!()
    }

    fn replacement(&self) -> Vec<u8> {
        unimplemented!()
    }
}

pub struct ShiftJISDecode;

impl EncodingStatics for ShiftJISDecode {
    fn new(_options: &str) -> Result<Box<dyn Encoding>, String> {
        Ok(Box::new(ShiftJISDecode))
    }

    fn print_help() {
        println!("Decodes Shift JIS into character data.");
        println!("(no options)");
    }
}

impl Encoding for ShiftJISDecode {
    fn next(&mut self, input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let mut bytes = vec![];

        let first_byte = match input.get_byte() {
            Some(Ok(byte)) => byte,
            Some(Err(e)) => { return Some(Err(e)); },
            None => { return None; }
        };
        bytes.push(first_byte);

        #[allow(clippy::match_overlapping_arm)]
        match first_byte {
            0x5C => {
                return Some(Ok(vec![0x00, 0x00, 0x00, 0xA5])); // YEN SIGN
            },
            0x7E => {
                return Some(Ok(vec![0x00, 0x00, 0x20, 0x3E])); // OVERLINE
            },
            0 ..= 0x7F => {
                debug!("{:#02x}: ASCII {}", first_byte, first_byte as char);
                return Some(Ok(vec![0x00, 0x00, 0x00, first_byte])); // un-altered ASCII
            },
            0x80 | 0xA0 | 0xF0 ..= 0xFF => {
                return Some(Err(CodeError::new("illegal first Shift JIS byte").with_bytes(bytes)));
            },
            0xA1 ..= 0xDF => {
                // Single-byte half-width katakana
                debug!("{:#02x}: single-byte half-width katakana", first_byte);
                return Some(Ok(utils::u32_to_bytes(HALFWIDTH[(first_byte - 0xA1) as usize] as u32, true)));
            },
            0x81 ..= 0x9F | 0xE0 ..= 0xEF => {
                // First byte of a double-byte JIS X 0208 character
                debug!("{:#x}: first byte of double-byte sequence", first_byte);
            },
        }

        let second_byte = match input.get_byte() {
            Some(Ok(byte)) => byte,
            Some(Err(e)) => {
                let err = CodeError::new("incomplete double-byte Shift JIS character")
                    .with_bytes(bytes)
                    .with_inner(e);
                return Some(Err(err));
            },
            None => {
                let err = CodeError::new("incomplete double-byte Shift JIS character due to EOF")
                    .with_bytes(bytes);
                return Some(Err(err));
            }
        };
        bytes.push(second_byte);

        match second_byte {
            0x00 ..= 0x3F | 0x7F | 0xFD ..= 0xFF => {
                let err = CodeError::new("illegal second byte of double-byte Shift JIS character")
                    .with_bytes(bytes);
                return Some(Err(err));
            },
            0x40 ..= 0x9E if first_byte % 2 == 0 => {
                let err = CodeError::new("mismatching second byte of double-byte Shift JIS character where first byte is even")
                    .with_bytes(bytes);
                return Some(Err(err));
            },
            0x9F ..= 0xFC if first_byte % 2 == 1 => {
                let err = CodeError::new("mismatching second byte of double-byte Shift JIS character where first byte is odd")
                    .with_bytes(bytes);
                return Some(Err(err));
            }
            _ => ()
        };

        debug!("{:#02x} {:#02x}", first_byte, second_byte);

        let mut j1 = match first_byte {
            0x81 ..= 0x9F => {
                (first_byte - 0x70) * 2
            },
            0xE0 ..= 0xEF => {
                (first_byte - 0xB0) * 2
            },
            _ => unreachable!()
        };
        
        let j2 = if second_byte < 0x9F {
            j1 -= 1;
            assert!(j1 % 2 == 1);
            if second_byte > 0x7E {
                second_byte - 0x20
            } else {
                second_byte - 0x1F
            }
        } else {
            second_byte - 0x7E
        };

        debug!("JIS sequence {:#02x} {:#02x}", j1, j2);

        // Now we have a JIS 0208 codepoint, how do we get a Unicode code point?

        //let codepoint: u16 = (j1 as u16) << 8 | (j2 as u16);
        //let unicode: u32 = encoding_index_japanese::jis0208::backward(codepoint as u32) as u32;
        //Some(Ok(utils::u32_to_bytes(unicode, true)))

        Some(Ok(self.replacement()))
    }

    fn replacement(&self) -> Vec<u8> {
        utils::unicode_replacement()
    }
}
