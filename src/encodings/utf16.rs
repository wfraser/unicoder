use super::utils;
use super::code_adapter::*;
use super::super::encoding::*;

pub struct Utf16 {
    adapter: U32Adapter,
}

impl CodeStatics for Utf16 {
    fn new(input: InputBox, options: &str) -> Result<InputBox, String> {
        let mut big_endian = false;
        match options.to_lowercase().as_str() {
            "le" => (),
            "be" => { big_endian = true; },
            _ => { return Err("utf16: invalid options".into()); }
        }

        Ok(Box::new(Utf16 {
            adapter: U32Adapter::new(input, true, Box::new(move |cp, out| process_codepoint(cp, out, big_endian))),
        }))
    }

    fn print_help() {
        println!("Encodes input character data as UTF-16.");
        println!("Options:");
        println!("  le = little endian (the default)");
        println!("  be = big endian");
    }
}

fn push_u16(codeunit: u16, out: &mut VecDequeWritable<u8>, big_endian: bool) {
    let hi = (codeunit >> 8) as u8;
    let lo = codeunit as u8;
    if big_endian {
        out.push(hi);
        out.push(lo);
    } else {
        out.push(lo);
        out.push(hi);
    }
}

fn process_codepoint(codepoint: u32, out: &mut VecDequeWritable<u8>, big_endian: bool) -> Result<(), CodeError> {
    // Note: UTF-16's names for the "high" and "low" surrogate are somewhat confusing.
    // "High" surrogates are in the range 0xD800 - 0xDBFF.
    // "Low" surrogates are in the range 0xDC00 - 0xDFFF.
    // The names are because the "low" surrogate encodes the 10 low-order bits of the code
    // point, and the "high" surrogate encodes the 10 high-order bits.

    if codepoint >= 0xD800 && codepoint <= 0xDFFF {
        // Forbidden range.
        let which = if codepoint < 0xDC00 {
            "low"
        } else {
            "high"
        };
        error!("cannot UTF-16 encode {} surrogate code point U+{:04X}", which, codepoint);
        Err(CodeError::new(format!("cannot UTF-16 encode {} surrogate code point", which))
                      .with_bytes(utils::u32_to_u8_be(codepoint)))
    } else if codepoint <= 0xFFFF {
        // Identity encoding.
        debug!("UTF-16 trivial encoding");
        push_u16(codepoint as u16, out, big_endian);
        Ok(())
    } else if codepoint <= 0x10_FFFF {
        // Surrogate pair encoding (codepoint >= 0x10000)
        debug!("UTF-16 surrogate pair encoding");
        let subtracted = codepoint - 0x1_0000;
        let high_surrogate = (0xD800 + (subtracted >> 10)) as u16;
        let low_surrogate = (0xDC00 + (subtracted & 0x3FF)) as u16;
        debug!("high = {:#X}", high_surrogate);
        debug!("low  = {:#X}", low_surrogate);
        push_u16(high_surrogate, out, big_endian);
        push_u16(low_surrogate, out, big_endian);
        Ok(())
    } else {
        // Codepoint > 0x10_FFFF
        error!("cannot UTF-16 encode out-of-range code point U+{:04X}", codepoint);
        Err(CodeError::new("cannot UTF-16 encode out-of-range code point")
                      .with_bytes(utils::u32_to_u8_be(codepoint)))
    }
}

impl Code for Utf16 {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        self.adapter.next()
    }
}
