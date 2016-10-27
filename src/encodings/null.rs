use super::super::encoding::*;

pub struct Null;

impl EncodingStatics for Null {
    fn new(_options: &str) -> Result<Box<Encoding>, String> {
        Ok(Box::new(Null))
    }

    fn print_help() {
        println!("Discards all input.");
        println!("(no options)");
    }
}

impl Encoding for Null {
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        while input.get_byte().is_some() {
            // do nothing
        }
        None
    }

    fn replacement(&self) -> Vec<u8> {
        vec![]
    }
}
