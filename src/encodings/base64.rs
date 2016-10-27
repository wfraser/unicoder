use super::super::encoding::*;

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
            for _ in 0 .. out.len() % 3 {
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
