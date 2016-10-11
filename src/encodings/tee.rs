use super::super::encoding::*;

use std::io::{self, Write};

pub struct Tee {
    input: InputBox,
}

impl CodeStatics for Tee {
    fn new(input: InputBox, _options: &str) -> InputBox {
        Box::new(Tee {
            input: input,
        }) as InputBox
    }

    fn print_help() {
        // TODO: add options for writing to arbitrary file descriptors or files instead of stdout
        println!("(no options)");
        println!("Copies its input verbatim to standard output.");
    }
}

impl Code for Tee {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        match self.input.next() {
            Some(Ok(byte)) => {
                if let Err(e) = io::stdout().write(&[byte]) {
                    error!("I/O error copying to stdout: {}", e);
                }
                Some(Ok(byte))
            },
            other => other,
        }
    }
}
