use super::super::encoding::*;

pub struct HexEncode {
    input: InputBox,
    uppercase: bool,
    output_buffer: Vec<u8>,
}

impl HexEncode {
    fn hex_chars(&self, byte: u8) -> (u8, u8) {
        debug!("byte = {:#x}", byte);
        (self.hex_char(byte >> 4), self.hex_char(byte & 0xF))
    }

    fn hex_char(&self, quad: u8) -> u8 {
        debug!("digit = {:#x}", quad);
        assert!(quad < 16);
        if quad < 10 {
            ('0' as u8) + quad
        } else if self.uppercase {
            ('A' as u8) + quad - 10
        } else {
            ('a' as u8) + quad - 10
        }
    }
}

impl CodeStatics for HexEncode {
    fn new(input: InputBox, _options: &str) -> InputBox {
        Box::new(HexEncode {
            input: input,
            uppercase: false,
            output_buffer: vec![],
        }) as Box<Code>
    }

    fn print_help() {
        println!("<hex help>");
    }
}

impl Code for HexEncode {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        if !self.output_buffer.is_empty() {
            return Some(Ok(self.output_buffer.pop().unwrap()));
        }

        match self.input.next() {
            Some(Ok(byte)) => {
                let (current, next) = self.hex_chars(byte);
                debug!("current = {:#x}, next = {:#x}", current, next);
                self.output_buffer.push(0x20);
                self.output_buffer.push(next);
                Some(Ok(current))
            },
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }
}

pub struct HexDecode {
    input: InputBox,
}

impl CodeStatics for HexDecode {
    fn new(input: InputBox, _options: &str) -> InputBox {
        Box::new(HexDecode {
            input: input,
        })
    }

    fn print_help() {
        println!("<unhex help>");
    }
}

impl Code for HexDecode {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        let mut out = 0u8;
        let mut first = true;

        loop {
            match self.input.next() {
                Some(Ok(byte)) => {
                    let c = byte as char;
                    let value = if c == ' ' || c == '\t' {
                        // skip whitespace
                        continue;
                    } else if c >= '0' && c <= '9' {
                        byte - ('0' as u8)
                    } else if c >= 'a' && c <= 'f' {
                        byte - ('a' as u8) + 10
                    } else if c >= 'A' && c <= 'F' {
                        byte - ('A' as u8) + 10
                    } else {
                        return Some(Err(CodeError::new("out of range")
                                                  .with_bytes([byte].to_vec())));
                    };
                    debug!("digit: {:x}", value);

                    if !first {
                        out <<= 4;
                    }

                    out += value;

                    if first {
                        first = false;
                    } else {
                        return Some(Ok(out));
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
