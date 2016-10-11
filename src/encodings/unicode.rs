use super::super::encoding::*;

use std::collections::VecDeque;
use std::io::{self, Write};

pub struct UnicodeInfo {
    input: InputBox,
    output_buffer: VecDequeWritable<u8>,
    stashed_error: Option<CodeError>,
}

struct VecDequeWritable<T> {
    inner: VecDeque<T>,
}

impl Write for VecDequeWritable<u8> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        for byte in buf {
            self.inner.push_front(*byte);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}

impl<T> VecDequeWritable<T> {
    fn new() -> VecDequeWritable<T> {
        VecDequeWritable {
            inner: VecDeque::new(),
        }
    }
    fn pop(&mut self) -> Option<T> {
        self.inner.pop_back()
    }
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl CodeStatics for UnicodeInfo {
    fn new(input: InputBox, _options: &str) -> InputBox {
        Box::new(UnicodeInfo {
            input: input,
            output_buffer: VecDequeWritable::new(),
            stashed_error: None,
        }) as InputBox
    }

    fn print_help() {
        println!("(no options)");
    }
}

impl UnicodeInfo {
    fn read_codepoint(input: &mut InputBox) -> Option<Result<u32, CodeError>> {
        let mut codepoint = 0u32;
        let mut bytes = Vec::with_capacity(4);
        for i in 0 .. 4 {
            match input.next() {
                Some(Ok(byte)) => {
                    debug!("UTF-32 byte {:02x}", byte);
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
                        return Some(Err(CodeError::new("incomplete UTF-32 code point")
                                                .with_bytes(bytes)));
                    }
                }
            }
        }
        Some(Ok(codepoint))
    }

    fn process_codepoint<W: Write>(codepoint: u32, out: &mut W) { // no result because it can't fail
        writeln!(out, "U+{:X} ", codepoint).unwrap();
        // TODO: write more info about the code point. Use `ucd` crate.
    }
}

impl Code for UnicodeInfo {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        if self.output_buffer.is_empty() {
            match Self::read_codepoint(&mut self.input) {
                Some(Ok(codepoint)) => {
                    Self::process_codepoint(codepoint, &mut self.output_buffer);
                },
                Some(Err(e)) => {
                    self.stashed_error = Some(e);
                },
                None => (),
            }
        }

        if let Some(byte) = self.output_buffer.pop() {
            return Some(Ok(byte));
        }

        if let Some(err) = self.stashed_error.take() {
            return Some(Err(err));
        }

        None
    }
}
