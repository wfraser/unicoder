use super::utils;
use super::super::encoding::*;

use std::collections::VecDeque;
use std::io::{self, Write};

pub struct Utf32Adapter {
    input: InputBox,
    f: Box<Fn(u32, &mut VecDequeWritable<u8>) -> Result<(), CodeError>>,
    output_buffer: VecDequeWritable<u8>,
    stashed_error: Option<CodeError>,
}

impl Utf32Adapter {
    pub fn new(input: InputBox, f: Box<Fn(u32, &mut VecDequeWritable<u8>) -> Result<(), CodeError>>) -> Utf32Adapter {
        Utf32Adapter {
            input: input,
            f: f,
            output_buffer: VecDequeWritable::new(),
            stashed_error: None,
        }
    }
}

impl Code for Utf32Adapter {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        if self.output_buffer.is_empty() {
            match utils::read_u32_be(&mut self.input) {
                Some(Ok(codepoint)) => {
                    debug!("got input: U+{:04x}", codepoint);
                    if let Err(e) = (self.f)(codepoint, &mut self.output_buffer) {
                        error!("processing function returned error: {}", e);
                        self.stashed_error = Some(e);
                    }
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

pub struct VecDequeWritable<T> {
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
    pub fn new() -> VecDequeWritable<T> {
        VecDequeWritable {
            inner: VecDeque::new(),
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_back()
    }
    pub fn push(&mut self, item: T) {
        self.inner.push_front(item)
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}
