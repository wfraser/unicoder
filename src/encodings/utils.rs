use super::super::encoding::*;

/// Read 4 bytes from the input, interpret as a big-endian 32-bit integer.
pub fn read_u32_be(input: &mut InputBox) -> Option<Result<u32, CodeError>> {
    let mut codepoint = 0u32;
    let mut bytes = Vec::with_capacity(4);
    for i in 0..4 {
        match input.next() {
            Some(Ok(byte)) => {
                debug!("UTF-32 byte {:02X}", byte);
                codepoint |= (byte as u32) << (8 * (3 - i));
                bytes.push(byte);
            },
            Some(Err(e)) => {
                return Some(Err(CodeError::new("incomplete UTF-32 code point")
                                          .with_bytes(bytes)
                                          .with_inner(e)));
            },
            None => {
                if i == 0 {
                    return None;
                } else {
                    return Some(Err(CodeError::new("incomplete UTF-32 code point at EOF")
                                              .with_bytes(bytes)));
                }
            }
        }
    }
    Some(Ok(codepoint))
}

/// Split a 32-bit integer into 4 bytes, big-endian (most-significant byte first).
pub fn u32_to_u8_be(input: u32) -> Vec<u8> {
    let mut out = Vec::with_capacity(4);
    out.push((input >> 24) as u8);
    out.push((input >> 16) as u8);
    out.push((input >> 8) as u8);
    out.push(input as u8);
    out
}
