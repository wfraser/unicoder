use super::super::encoding::*;

#[derive(Debug)]
pub struct Base64 {
    pub code62: u8,
    pub code63: u8,
    pub pad: Option<u8>,
}

impl Base64 {
    fn encode_internal(&self, n: u8) -> u8 {
        match n {
            0 ... 25 => b'A' + n,
            26 ... 51 => b'a' + (n - 26),
            52 ... 61 => b'0' + (n - 52),
            62 => self.code62,
            63 => self.code63,
            _ => panic!("out-of-range input to Base64::encode_internal")
        }
    }

    fn decode_internal(&self, n: u8) -> Result<u8, String> {
        if n == self.code62 {
            Ok(62)
        } else if n == self.code63 {
            Ok(63)
        } else {
            match n {
                b'A' ... b'Z' => Ok(n - b'A'),
                b'a' ... b'z' => Ok(n - b'a' + 26),
                b'0' ... b'9' => Ok(n - b'0' + 52),
                _ => Err(format!("invalid Base64 character {:#04X}", n))
            }
        }
    }

    pub fn encode(&self, bytes: &[u8]) -> Vec<u8> {
        let mut out = vec![];
        debug!("in: {:?}", bytes);

        // 0: abcdefgh
        //    11111100              & 0xFC >> 2
        // 1: abcdefgh ijklmnop
        //    00000011 11110000     & 0x03 << 4, & 0xF0 >> 4, advance
        // 2: ijklmnop qrstuvwx
        //    00001111 11000000     & 0x0F << 2, & 0xC0 >> 6, advance
        // 3: qrstuvwx
        //    00111111              & 0x3F, advance

        let mut state = 0;
        let mut pos = 0;
        while pos < bytes.len() {
            let n = match state % 4 {
                0 => (bytes[pos] & 0xFC) >> 2,
                1 => {
                    let n = ((bytes[pos] & 0x03) << 4) | ((bytes.get(pos+1).unwrap_or(&0) & 0xF0) >> 4);
                    pos += 1;
                    n
                },
                2 => {
                    let n = ((bytes[pos] & 0x0F) << 2) | ((bytes.get(pos+1).unwrap_or(&0) & 0xC0) >> 6);
                    pos += 1;
                    n
                },
                3 => {
                    let n = bytes[pos] & 0x3F;
                    pos += 1;
                    n
                },
                _ => unreachable!()
            };
            out.push(self.encode_internal(n));
            if state == 3 && pos == bytes.len() {
                break;
            }
            state += 1;
        }

        if let Some(byte) = self.pad {
            let npad = ((out.len() + 3) & !3) - out.len();
            debug!("{} output chars, adding {} padding", out.len(), npad);
            assert!(npad != 3);
            for _ in 0 .. npad {
                out.push(byte);
            }
        }

        debug!("out = {:?}", out);
        out
    }

    pub fn decode(&self, bytes: &[u8]) -> Result<Vec<u8>, (Vec<u8>, CodeError)> {
        let mut out = vec![];

        let data_len = if let Some(pad) = self.pad {
            if bytes[bytes.len() - 1] == pad {
                if bytes[bytes.len() - 2] == pad {
                    bytes.len() - 2
                } else {
                    bytes.len() - 1
                }
            } else {
                bytes.len()
            }
        } else {
            bytes.len()
        };

        let mut partial = 0u8;
        for (i, byte) in bytes[0..data_len].iter().enumerate() {
            let value = match self.decode_internal(*byte) {
                Ok(value) => value,
                Err(e) => {
                    return Err((out, CodeError::new(e)));
                }
            };

            // 0: abcdefgh
            //    11111100              << 2
            // 1: abcdefgh ijklmnop
            //    00000011 11110000     & 30 >> 4; & 0F << 4
            // 2: ijklmnop qrstuvwx
            //    00001111 11000000     & 3C >> 2; & 03 << 6
            // 3: qrstuvwx
            //    00111111              & 3F

            debug!("value: {:#04X}", value);
            match i % 4 {
                0 => {
                    partial = value << 2;
                },
                1 => {
                    partial |= (value & 0x30) >> 4;
                    debug!("out: {:#04X}", partial);
                    out.push(partial);
                    partial = (value & 0x0F) << 4;
                },
                2 => {
                    partial |= (value & 0x3C) >> 2;
                    debug!("out: {:#04X}", partial);
                    out.push(partial);
                    partial = (value & 0x03) << 6;
                },
                3 => {
                    partial |= value & 0x3F;
                    debug!("out: {:#04X}", partial);
                    out.push(partial);
                }
                _ => unreachable!()
            }
        }

        if self.pad.is_none() && data_len % 4 != 0 {
            debug!("out: {:#02X}", partial);
            out.push(partial);
        }

        Ok(out)
    }
}

pub struct Base64Encode {
    base64: Base64,
    line_width: Option<usize>,
    output_line_width: usize,
}

fn parse_single_byte(s: &str) -> Result<u8, String> {
    if s.len() > 1 {
        Err(format!("argument must be a single character, not {:?}", s))
    } else {
        let c = s.chars().next();
        if c.is_none() {
            Err("argument must be a single character, not empty".into())
        } else {
            let c = c.unwrap();
            if (c as u32) > 0xFF {
                Err(format!("argument needs to fit in a single byte (0-255), not {}", c as u32))
            } else {
                Ok((c as u32) as u8)
            }
        }
    }
}

impl EncodingStatics for Base64Encode {
    fn new(options: &str) -> Result<Box<Encoding>, String> {
        let mut code62 = b'+';
        let mut code63 = b'/';
        let mut pad = Some(b'=');
        let mut width = Some(64);

        if options != "" {
            for arg in options.split(',') {
                let parts: Vec<&str> = arg.split('=').collect();
                match parts[0] {
                    "62" => { code62 = try!(parse_single_byte(parts[1])); },
                    "63" => { code63 = try!(parse_single_byte(parts[1])); },
                    "padding" => {
                        if parts[1] == "none" {
                            pad = None;
                        } else {
                            pad = Some(try!(parse_single_byte(parts[1])));
                        }
                    },
                    "width" => {
                        if parts[1] == "none" {
                            width = None;
                        } else {
                            width = match parts[1].parse() {
                                Ok(w) => Some(w),
                                Err(e) => {
                                    return Err(format!("width must be a number: {}", e));
                                }
                            };
                        }
                    }
                    _ => {
                        return Err(format!("unrecognized argument {:?}", arg));
                    }
                }
            }
        }
        debug!("base64 settings: 62={:?} 63={:?} pad={:?}", code62 as char, code63 as char, pad.map(|c| c as char));

        Ok(Box::new(Base64Encode {
            base64: Base64 {
                code62: code62,
                code63: code63,
                pad: pad,
            },
            line_width: width,
            output_line_width: 0,
        }))
    }

    fn print_help() {
        println!("Encodes data as Base64.");
        println!("Options:");
        println!("  62=<character>      Which character to encode 62 as? (default: '+')");
        println!("  63=<character>      Which character to encode 63 as? (default: '/')");
        println!("  padding=<character> Which character to use for padding? (default: '=')");
        println!("                          Can also be set to 'none' to disable padding.");
        println!("  width=<line width>  How long to make lines before breaking with \"<CR><LF>\"?");
        println!("                          Default is 64. Can also be set to 'none' to disable wrapping.");
    }
}

impl Encoding for Base64Encode {
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let mut bytes = Vec::<u8>::new();
        for i in 0..3 {
            match input.get_byte() {
                Some(Ok(byte)) => { bytes.push(byte); },
                Some(Err(e)) => { return Some(Err(CodeError::new("error getting byte")
                                                            .with_bytes(bytes)
                                                            .with_inner(e))); },
                None => {
                    if i == 0 {
                        return None;
                    } else {
                        break;
                    }
                },
            };
        }
        debug!("encoding {} bytes", bytes.len());

        let encoded = self.base64.encode(&bytes);

        if let Some(line_width) = self.line_width {
            if encoded.len() + self.output_line_width >= line_width {
                let mut out = vec![];
                out.extend_from_slice(&encoded[0 .. line_width - self.output_line_width]);
                out.push(b'\n');
                out.extend_from_slice(&encoded[line_width - self.output_line_width ..]);
                self.output_line_width = encoded.len() - (line_width - self.output_line_width);
                Some(Ok(out))
            } else {
                self.output_line_width += encoded.len();
                Some(Ok(encoded))
            }
        } else {
            Some(Ok(encoded))
        }
    }
}
