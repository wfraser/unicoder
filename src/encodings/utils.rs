pub fn u32_from_bytes(bytes: &[u8], big_endian: bool) -> u32 {
    let mut out = 0u32;
    for i in 0 .. 4 {
        let shift = if big_endian {
            3 - i
        } else {
            i
        };
        out |= (bytes[i] as u32) << (8 * shift);
    }
    out
}

pub fn u32_to_bytes(input: u32, big_endian: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(4);
    for i in 0 .. 4 {
        let shift = if big_endian {
            3 - i
        } else {
            i
        };
        out.push((input >> (8 * shift)) as u8);
    }
    out
}

pub fn u16_to_bytes(input: u16, big_endian: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(2);
    let hi = (input >> 8) as u8;
    let lo = input as u8;
    if big_endian {
        out.push(hi);
        out.push(lo);
    } else {
        out.push(lo);
        out.push(hi);
    }
    out
}

pub fn unicode_replacement() -> Vec<u8> {
    // Basically this:
    //u32_to_bytes(::std::char::REPLACEMENT_CHARACTER as u32, true)
    vec![0, 0, 0xFF, 0xFD]
}
