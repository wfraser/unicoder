use super::super::encoding::*;
use super::utils;

use std::char;
use std::fmt::Debug;

use ucd::*;

pub struct UnicodeInfo;

impl EncodingStatics for UnicodeInfo {
    fn new(_options: &str) -> Result<Box<dyn Encoding>, String> {
        Ok(Box::new(UnicodeInfo))
    }

    fn print_help() {
        println!("Displays Unicode character info for UTF-32BE input. Doesn't transform the data.");
        println!("(no options)");
    }
}

fn name<T: Debug>(x: &T) -> String {
    let raw = format!("{:?}", x);
    let mut out = String::new();
    let mut first = true;
    for c in raw.chars() {
        if c.is_uppercase() {
            if !first {
                out.push(' ');
            }
            out.push(c);
        } else {
            out.push(c);
        }
        first = false;
    }
    out
}

impl Encoding for UnicodeInfo {
    fn next(&mut self, input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let bytes = match input.get_bytes(4) {
            Some(Ok(bytes)) => bytes,
            Some(Err(e)) => { return Some(Err(e)); },
            None => { return None; },
        };

        let codepoint = utils::u32_from_bytes(&bytes, true);
        println!("U+{:04X}: {}", codepoint, unicode_name(codepoint));

        let c = unsafe { char::from_u32_unchecked(codepoint) };
        println!("\tblock:    {}", c.block().map(|x| name(&x)).unwrap_or_else(|| "none".into()));
        println!("\tcategory: {}", name(&c.category()));
        println!("\tscript:   {}", c.script().map(|x| name(&x)).unwrap_or_else(|| "none".into()));

        Some(Ok(bytes))
    }
}

fn unicode_name(codepoint: u32) -> String {
    if codepoint > 0x10FFFF {
        // unicode_names doesn't play nicely with these
        return "(out of Unicode range)".to_string();
    }

    // Some names for characters not in the database.
    let alt_name = match codepoint {
        // The C0 and C1 control codes have a name of "<control>" which isn't returned by the
        // unicode_names crate, but nearly all of them have an official "alias" name, so use that.
        // C0 controls
        0x0 => "NULL",
        0x1 => "START OF HEADING",
        0x2 => "START OF TEXT",
        0x3 => "END OF TEXT",
        0x4 => "END OF TRANSMISSION",
        0x5 => "ENQUIRY",
        0x6 => "ACKNOWLEDGE",
        0x7 => "BELL",
        0x8 => "BACKSPACE",
        0x9 => "CHARACTER TABULATION",
        0xA => "LINE FEED",
        0xB => "LINE TABULATION",
        0xC => "FORM FEED",
        0xD => "CARRIAGE RETURN",
        0xE => "SHIFT OUT",
        0xF => "SHIFT IN",
        0x10 => "DATA LINK ESCAPE",
        0x11 => "DEVICE CONTROL ONE",
        0x12 => "DEVICE CONTROL TWO",
        0x13 => "DEVICE CONTROL THREE",
        0x14 => "DEVICE CONTROL FOUR",
        0x15 => "NEGATIVE ACKNOWLEDGE",
        0x16 => "SYNCHRONOUS IDLE",
        0x17 => "END OF TRANSMISSION BLOCK",
        0x18 => "CANCEL",
        0x19 => "END OF MEDIUM",
        0x1A => "SUBSTITUTE",
        0x1B => "ESCAPE",
        0x1C => "INFORMATION SEPARATOR FOUR",
        0x1D => "INFORMATION SEPARATOR THREE",
        0x1E => "INFORMATION SEPARATOR TWO",
        0x1F => "INFORMATION SEPARATOR ONE",
        0x7F => "DELETE",
        // C1 controls
        // 0x80 = ?
        // 0x81 = ?
        0x82 => "BREAK PERMITTED HERE",
        0x83 => "NO BREAK HERE",
        0x84 => "formerly known as INDEX",
        0x85 => "NEXT LINE (NEL)",
        0x86 => "START OF SELECTED AREA",
        0x87 => "END OF SELECTED AREA",
        0x88 => "CHARACTER TABULATION SET",
        0x89 => "CHARACTER TABULATION WITH JUSTIFICATION",
        0x8A => "LINE TABULATION SET",
        0x8B => "PARTIAL LINE FORWARD",
        0x8C => "PARTIAL LINE BACKWARD",
        0x8D => "REVERSE LINE FEED",
        0x8E => "SINGLE SHIFT TWO",
        0x8F => "SINGLE SHIFT THREE",
        0x90 => "DEVICE CONTROL STRING",
        0x91 => "PRIVATE USE ONE",
        0x92 => "PRIVATE USE TWO",
        0x93 => "SET TRANSMIT STATE",
        0x94 => "CANCEL CHARACTER",
        0x95 => "MESSAGE WAITING",
        0x96 => "START OF GUARDED AREA",
        0x97 => "END OF GUARDED AREA",
        0x98 => "START OF STRING",
        // 0x99 = ?
        0x9A => "SINGLE CHARACTER INTRODUCER",
        0x9B => "CONTROL SEQUENCE INTRODUCER",
        0x9C => "STRING TERMINATOR",
        0x9D => "OPERATING SYSTEM COMMAND",
        0x9E => "PRIVACY MESSAGE",
        0x9F => "APPLICATION PROGRAM COMMAND",

        // Surrogates
        0x00D800 ..= 0x00DBFF => "<high surrogate>",
        0x00DC00 ..= 0x00DCFF => "<low surrogate>",
        // Private use
        0x00E000 ..= 0x00F8FD |                     // Private Use Area
        0x0F0000 ..= 0x0FFFFD |                     // Supplementary Private Use Area-A
        0x100000 ..= 0x10FFFD => "<private use>",   // Supplementary Private Use Area-B
        // Non-characters
        0x00FDD0 ..= 0x00FDEF => "<not a character>",

        // This is much more often used as a BOM than as a ZWNBSP
        0x00FEFF => "<byte order mark>",

        // Likewise, if the endian-ness is set wrong, this is what shows up for a BOM.
        0x00FFFE => "<not a character> (swapped byte order mark)",

        // U+XYFFFE and U+XYFFFF are non-characters for all X and Y.
        other if other & 0xFFFE == 0xFFFE => "<not a character>",
        _ => ""
    };

    if !alt_name.is_empty() {
        return alt_name.to_string();
    }

    let c = unsafe { char::from_u32_unchecked(codepoint) };
    match unicode_names::name(c) {
        Some(name) => name.to_string(),
        None => "(unknown character)".to_string(),
    }
}
