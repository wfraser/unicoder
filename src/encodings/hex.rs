use super::super::encoding::*;

pub struct HexEncode {
    uppercase: bool,
    done: bool,
}

impl HexEncode {
    fn hex_chars(&self, byte: u8) -> (u8, u8) {
        debug!("byte = {:#04X}", byte);
        (self.hex_char(byte >> 4), self.hex_char(byte & 0xF))
    }

    fn hex_char(&self, quad: u8) -> u8 {
        assert!(quad < 16);
        if quad < 10 {
            b'0' + quad
        } else if self.uppercase {
            b'A' + quad - 10
        } else {
            b'a' + quad - 10
        }
    }
}

impl EncodingStatics for HexEncode {
    fn new(_options: &str) -> Result<Box<dyn Encoding>, String> {
        Ok(Box::new(HexEncode {
            uppercase: false,
            done: false,
        }))
    }

    fn print_help() {
        // TODO: add options for spacing, upper/lowercase, etc.
        println!("Formats input as hexadecimal, 2 digits, space separated.");
        println!("(no options)");
    }
}

impl Encoding for HexEncode {
    fn next(&mut self, input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        match input.get_byte() {
            Some(Ok(byte)) => {
                let (high, low) = self.hex_chars(byte);
                Some(Ok(vec![high, low, b' ']))
            },
            Some(Err(e)) => Some(Err(e)),
            None => {
                if self.done {
                    self.done = false;
                    None
                } else {
                    self.done = true;
                    Some(Ok(vec![b'\n']))
                }
            }
        }
    }
}

pub struct HexDecode;

impl EncodingStatics for HexDecode {
    fn new(_options: &str) -> Result<Box<dyn Encoding>, String> {
        Ok(Box::new(HexDecode))
    }

    fn print_help() {
        println!("Parses hexadecimal input into raw data. Ignores whitespace.");
        println!("(no options)");
    }
}

impl Encoding for HexDecode {
    fn next(&mut self, input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let mut out = 0u8;
        let mut first = true;

        loop {
            match input.get_byte() {
                Some(Ok(byte)) => {
                    let c = byte as char;
                    let value = if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
                        // skip whitespace
                        continue;
                    } else if ('0'..='9').contains(&c) {
                        byte - b'0'
                    } else if ('a'..='f').contains(&c) {
                        byte - b'a' + 10
                    } else if ('A'..='F').contains(&c) {
                        byte - b'A' + 10
                    } else {
                        error!("out of range: {:?}", c);
                        return Some(Err(CodeError::new("out of range")
                                                  .with_bytes([byte].to_vec())));
                    };
                    debug!("read digit {:X}", value);

                    if !first {
                        out <<= 4;
                    }

                    out += value;

                    if first {
                        first = false;
                    } else {
                        return Some(Ok(vec![out]));
                    }
                },
                Some(Err(e)) => {
                    return Some(Err(e));
                },
                None => {
                    if !first {
                        error!("not enough data (need a second hex digit to finish the octet): {:#x}X", out);
                        return Some(Err(CodeError::new("not enough data (need a second hex digit to finish the octet)")
                                                  .with_bytes([out].to_vec())));
                    } else {
                        return None;
                    }
                }
            }
        }
    }
}
