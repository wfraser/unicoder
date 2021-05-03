use std::convert::TryInto;

pub fn u32_from_bytes(bytes: &[u8], big_endian: bool) -> u32 {
    let arr = bytes.split_at(4).0.try_into().expect("not enough bytes");
    if big_endian {
        u32::from_be_bytes(arr)
    } else {
        u32::from_le_bytes(arr)
    }
}

pub fn u32_to_bytes(input: u32, big_endian: bool) -> Vec<u8> {
    let arr = if big_endian {
        input.to_be_bytes()
    } else {
        input.to_le_bytes()
    };
    arr.to_vec()
}

pub fn u16_to_bytes(input: u16, big_endian: bool) -> Vec<u8> {
    let arr = if big_endian {
        input.to_be_bytes()
    } else {
        input.to_le_bytes()
    };
    arr.to_vec()
}

pub fn u16_from_bytes(bytes: &[u8], big_endian: bool) -> u16 {
    let arr = bytes.split_at(2).0.try_into().expect("not enough bytes");
    if big_endian {
        u16::from_be_bytes(arr)
    } else {
        u16::from_le_bytes(arr)
    }
}

pub fn unicode_replacement() -> Vec<u8> {
    // Basically this:
    //u32_to_bytes(::std::char::REPLACEMENT_CHARACTER as u32, true)
    vec![0, 0, 0xFF, 0xFD]
}
