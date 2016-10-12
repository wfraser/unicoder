use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct CodeError {
    message: String,
    bad_bytes: Option<Vec<u8>>,
    inner: Option<Box<Error>>,
}

impl CodeError {
    pub fn new<S: Into<String>>(message: S) -> CodeError {
        CodeError {
            message: message.into(),
            bad_bytes: None,
            inner: None,
        }
    }

    pub fn with_bytes(mut self, bytes: Vec<u8>) -> CodeError {
        self.bad_bytes = Some(bytes);
        self
    }

    pub fn with_inner<E: Error + 'static>(mut self, inner: E) -> CodeError {
        self.inner = Some(Box::new(inner) as Box<Error>);
        self
    }
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "Encoding Error: {}", self.message));
        if let Some(ref bytes) = self.bad_bytes {
            try!(write!(f, " ([{:x}", bytes[0]));
            for byte in bytes.iter().skip(1) {
                try!(write!(f, ",{:x}", byte));
            }
            try!(write!(f, "])"));
        }
        if let Some(ref e) = self.inner {
            try!(write!(f, " due to {}", e));
        }
        Ok(())
    }
}

impl Error for CodeError {
    fn description(&self) -> &str {
        "encoding error"
    }

    fn cause(&self) -> Option<&Error> {
        match self.inner {
            Some(ref innerbox) => Some(innerbox.as_ref()),
            None => None,
        }
    }
}

pub trait Code {
    fn next(&mut self) -> Option<Result<u8, CodeError>>;
}

pub trait CodeStatics {
    fn new(input: InputBox, options: &str) -> Result<InputBox, String>;
    fn print_help();
}

impl Iterator for Code {
    type Item = Result<u8, CodeError>;
    fn next(&mut self) -> Option<Self::Item> {
        (self as &mut Code).next()
    }
}

pub type InputBox = Box<Code>;
