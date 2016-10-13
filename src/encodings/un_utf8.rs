use super::super::encoding::*;

use std::collections::VecDeque;

pub struct UnUtf8 {
    input: InputBox,
    output_buffer: VecDeque<u8>,
    input_buffer: VecDeque<u8>, // sometimes it's necessary to push a byte back to the input
}

impl CodeStatics for UnUtf8 {
    fn new(input: InputBox, _options: &str) -> Result<InputBox, String> {
        Ok(Box::new(UnUtf8 {
            input: input,
            output_buffer: VecDeque::new(),
            input_buffer: VecDeque::new(),
        }))
    }

    fn print_help() {
        // TODO: add a "strict" mode that checks for overlong sequences (or maybe invert that and
        // have a "relaxed" mode)
        // TODO: add a mode that yields substitution characters instead of errors
        println!("Decodes UTF-8 input into character data (UTF-32BE)");
        println!("(no options)");
    }
}

impl UnUtf8 {
    fn push_codepoint(&mut self, codepoint: u32) {
        debug!("pushing codepoint U+{:X}", codepoint);
        self.output_buffer.push_back(((codepoint >> 24) & 0xFF) as u8);
        self.output_buffer.push_back(((codepoint >> 16) & 0xFF) as u8);
        self.output_buffer.push_back(((codepoint >> 8) & 0xFF) as u8);
        self.output_buffer.push_back((codepoint & 0xFF) as u8);
    }

    fn handle_initial_byte(&mut self, byte: u8) -> Result<(u32, u8), CodeError> {
        let codepoint: u32;
        let nbytes: u8;
        if byte < 0b10000000 {
            // 1 byte
            debug!("{:#x}: single byte sequence", byte);
            return Ok((byte as u32, 1));
        } else if byte < 0b11100000 {
            // 2 bytes:
            // 110ABCDE 10FGHIJK
            codepoint = ((byte & 0b00011111) as u32) << 6;
            nbytes = 2;
        } else if byte < 0b11110000 {
            // 3 bytes:
            // 1110ABCD 10EFGHIJ 10KLMNOP
            codepoint = ((byte & 0b00001111) as u32) << (6 * 2);
            nbytes = 3;
        } else if byte < 0b11111000 {
            // 4 bytes:
            // 11110ABC 10DEFGHI 10JKLMNO 10PQRSTU
            codepoint = ((byte & 0b00000111) as u32) << (6 * 3);
            nbytes = 4;
        } else if byte < 0b11111100 {
            // 5 bytes:
            // 111110AB 10CDEFGH 10IJKLMN 10OPQRST 10UVWXYZ
            codepoint = ((byte & 0b00000011) as u32) << (6 * 4);
            nbytes = 5;
        } else if byte < 0b11111110 {
            // 6 bytes:
            // 1111110A 10BCDEFG 10HIJKLM 10NOPQRS 10TUVWXY 10ZABCDE
            codepoint = ((byte & 0b00000001) as u32) << (6 * 5);
            nbytes = 6;
        } else {
            // byte == 0b11111111 or 0b11111110
            error!("illegal byte {:#x}", byte);
            return Err(CodeError::new("illegal byte")
                                 .with_bytes([byte].to_vec()));
        }
        debug!("{:#x}: initial byte of {}-byte sequence", byte, nbytes);
        debug!("{:032b}, {} bytes, shift = {}, buffer = {:?}", codepoint, nbytes, 6 * (nbytes - 1), &[byte]);
        Ok((codepoint, nbytes))
    }

    fn parse_input(&mut self) -> Result<(), CodeError> {
        let mut codepoint = 0u32;
        let mut nbytes = 0;
        let mut bytes = vec![];

        loop {
            let byte = if !self.input_buffer.is_empty() {
                self.input_buffer.pop_front().unwrap()
            } else {
                match self.input.next() {
                    Some(Ok(byte)) => byte,
                    Some(Err(e)) => {
                        return Err(e);
                    },
                    None => {
                        return if bytes.is_empty() {
                            Ok(())
                        } else {
                            error!("incomplete code point at EOF: expected {} bytes, only got {:?}", nbytes, bytes);
                            Err(CodeError::new("incomplete code point at EOF")
                                          .with_bytes(bytes))
                        };
                    }
                }
            };

            if nbytes != 0 {
                if byte < 0b10000000 {
                    error!("incomplete multi-byte code point: expected {} bytes, only got {:?}, due to single-byte codepoint {:#x}",
                        nbytes, bytes, byte);

                    // Since the read byte is a complete codepoint, might as well push it to the
                    // output buffer now.
                    self.push_codepoint(byte as u32);

                    return Err(CodeError::new("incomplete multi-byte code point")
                                         .with_bytes(bytes));
                } else if byte >= 0b11000000 {
                    error!("illegal initial byte {} when continuation byte expected: expected {} bytes, only got {:?}",
                        byte, nbytes, bytes);

                    // put this byte back in the input; deal with it next time.
                    self.input_buffer.push_back(byte);

                    return Err(CodeError::new("illegal initial byte when continuation byte expected")
                                         .with_bytes(bytes));
                } else {
                    // Continuation byte.
                    let shift = 6 * (nbytes - bytes.len() as u8 - 1);
                    codepoint |= ((byte & 0b00111111) as u32) << shift;
                    debug!("{:#x}: continuation byte", byte);
                    debug!("{:032b}, {} bytes, shift = {}, buffer = {:?}", codepoint, nbytes, shift, bytes);
                    if shift == 0 {
                        // We just read the last byte in the sequence.
                        if (nbytes == 2 && codepoint < 0x80)
                                || (nbytes == 3 && codepoint < 0x2080)
                                || (nbytes == 4 && codepoint < 0x82080)
                                || nbytes > 4 {
                            warn!("overlong sequence: {:?}", bytes);
                            // TODO: make it configurable to raise an error here.
                        }
                        self.push_codepoint(codepoint);
                        return Ok(())
                    } else {
                        bytes.push(byte);
                    }
                }
            }

            if nbytes == 0 {
                // Expecting an initial byte.
                match self.handle_initial_byte(byte) {
                    Ok((new_codepoint, new_nbytes)) => {
                        if new_nbytes == 1 {
                            // Single-byte codepoint. Push it and return.
                            self.push_codepoint(new_codepoint);
                            return Ok(());
                        } else {
                            // Start of a multi-byte sequence.
                            codepoint = new_codepoint;
                            nbytes = new_nbytes;
                            // and loop again.
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    },
                }
            }
        }
    }
}

impl Code for UnUtf8 {
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
