use super::super::encoding::*;

use std::mem;

pub struct UnUtf8 {
    input: InputBox,
    output_buffer: Vec<u8>,
    input_buffer: Vec<u8>,
    nbytes: u8,
    codepoint: u32,
}

impl CodeStatics  for UnUtf8 {
    fn new(input: InputBox, _options: &str) -> InputBox {
        Box::new(UnUtf8 {
            input: input,
            output_buffer: vec![],
            input_buffer: vec![],
            nbytes: 0,
            codepoint: 0,
        }) as InputBox
    }

    fn print_help() {
        println!("<unutf8 help>");
    }
}

impl UnUtf8 {
    // return the most-significant byte and push the rest to the output buffer in order.
    fn return_codepoint(&mut self, codepoint: u32) -> u8 {
        debug!("returning codepoint U+{:X}", codepoint);
        self.output_buffer.push((codepoint & 0xFF) as u8);
        self.output_buffer.push(((codepoint >> 8) & 0xFF) as u8);
        self.output_buffer.push(((codepoint >> 16) & 0xFF) as u8);
        return ((codepoint >> 24) & 0xFF) as u8;
    }

    fn push_codepoint(&mut self, codepoint: u32) {
        debug!("pushing codepoint U+{:X}", codepoint);
        self.output_buffer.push((codepoint & 0xFF) as u8);
        self.output_buffer.push(((codepoint >> 8) & 0xFF) as u8);
        self.output_buffer.push(((codepoint >> 16) & 0xFF) as u8);
        self.output_buffer.push(((codepoint >> 24) & 0xFF) as u8);
    }

    fn handle_initial_byte(&mut self, byte: u8) -> Option<Result<u32, CodeError>> {
        if byte < 0b10000000 {
            // 1 byte
            debug!("{:#x}: single byte sequence", byte);
            return Some(Ok(byte as u32));
        } else if byte < 0b11100000 {
            // 2 bytes:
            // 110ABCDE 10FGHIJK
            self.codepoint = ((byte & 0b00011111) as u32) << 6;
            self.nbytes = 2;
        } else if byte < 0b11110000 {
            // 3 bytes:
            // 1110ABCD 10EFGHIJ 10KLMNOP
            self.codepoint = ((byte & 0b00001111) as u32) << (6 * 2);
            self.nbytes = 3;
        } else if byte < 0b11111000 {
            // 4 bytes:
            // 11110ABC 10DEFGHI 10JKLMNO 10PQRSTU
            self.codepoint = ((byte & 0b00000111) as u32) << (6 * 3);
            self.nbytes = 4;
        } else if byte < 0b11111100 {
            // 5 bytes:
            // 111110AB 10CDEFGH 10IJKLMN 10OPQRST 10UVWXYZ
            self.codepoint = ((byte & 0b00000011) as u32) << (6 * 4);
            self.nbytes = 5;
        } else if byte < 0b11111110 {
            // 6 bytes:
            // 1111110A 10BCDEFG 10HIJKLM 10NOPQRS 10TUVWXY 10ZABCDE
            self.codepoint = ((byte & 0b00000001) as u32) << (6 * 5);
            self.nbytes = 6;
        } else {
            // byte == 0b11111111 or 0b11111110
            error!("illegal byte {:#x}", byte);
            return Some(Err(CodeError::new("illegal byte").with_bytes([byte].to_vec())));
        }
        debug!("{:#x}: initial byte of {}-byte sequence", byte, self.nbytes);
        self.input_buffer.push(byte);
        debug!("{:032b}, {} bytes, shift = {}, buffer = {:?}", self.codepoint, self.nbytes, 6 * (self.nbytes - 1), self.input_buffer);
        None
    }
}

impl Code for UnUtf8 {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        match self.output_buffer.pop() {
            Some(byte) => return Some(Ok(byte)),
            None => (),
        }

        loop {
            match self.input.next() {
                Some(Ok(byte)) => {
                    if self.nbytes != 0 {
                        if byte < 0b10000000 {
                            self.push_codepoint(byte as u32);
                            self.nbytes = 0;
                            error!("incomplete multi-byte code point: expected {} bytes, only got {:?}",
                                self.nbytes, self.input_buffer);
                            let err = CodeError::new("incomplete multi-byte code point")
                                                .with_bytes(mem::replace(&mut self.input_buffer, vec![]));
                            return Some(Err(err));
                        } else if byte >= 0b11000000 {
                            self.handle_initial_byte(byte);
                            error!("illegal initial byte {} when continuation byte expected: expected {} bytes, only got {:?}",
                                byte, self.nbytes, self.input_buffer);
                            let err = CodeError::new("illegal initial byte when continuation byte expected")
                                                .with_bytes(mem::replace(&mut self.input_buffer, vec![]));
                            return Some(Err(err));
                        } else {
                            let shift = 6 * (self.nbytes - self.input_buffer.len() as u8 - 1);
                            self.codepoint |= ((byte & 0b00111111) as u32) << shift;
                            debug!("{:#x}: continuation byte", byte);
                            debug!("{:032b}, {} bytes, shift = {}, buffer = {:?}", self.codepoint, self.nbytes, shift, self.input_buffer);
                            if shift == 0 {
                                if (self.nbytes == 2 && self.codepoint < 0x80)
                                        || (self.nbytes == 3 && self.codepoint < 0x2080)
                                        || (self.nbytes == 4 && self.codepoint < 0x82080)
                                        || self.nbytes > 4 {
                                    warn!("overlong sequence: {:?}", self.input_buffer);
                                    // TODO: make it configurable to raise an error here.
                                }
                                self.input_buffer.clear();
                                self.nbytes = 0;
                                let codepoint = self.codepoint;
                                self.codepoint = 0;
                                return Some(Ok(self.return_codepoint(codepoint)));
                            } else {
                                self.input_buffer.push(byte);
                            }
                        }
                    }

                    if self.nbytes == 0 {
                        match self.handle_initial_byte(byte) {
                            Some(Ok(codepoint)) => {
                                return Some(Ok(self.return_codepoint(codepoint)));
                            },
                            Some(Err(e)) => {
                                return Some(Err(e));
                            },
                            None => (),
                        }
                    }
                },
                Some(Err(e)) => {
                    return Some(Err(e));
                },
                None => {
                    return if self.input_buffer.is_empty() {
                        None
                    } else {
                        error!("incomplete code point at EOF: expected {} bytes, only got {:?}",
                            self.nbytes, self.input_buffer);
                        Some(Err(CodeError::new("incomplete code point at EOF")
                                           .with_bytes(mem::replace(&mut self.input_buffer, vec![]))))
                    }
                }
            }
        }
    }
}
