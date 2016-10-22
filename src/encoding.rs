use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

/// An error in the encoding/decoding process.
#[derive(Debug)]
pub struct CodeError {
    message: String,
    bad_bytes: Option<Vec<u8>>,
    inner: Option<Box<Error>>,
    encoding_name: Option<String>,
}

impl CodeError {
    /// Creates a new error with the given message.
    pub fn new<S: Into<String>>(message: S) -> CodeError {
        CodeError {
            message: message.into(),
            bad_bytes: None,
            inner: None,
            encoding_name: None,
        }
    }

    /// Include the bytes leading up to the error.
    pub fn with_bytes(mut self, bytes: Vec<u8>) -> CodeError {
        self.bad_bytes = Some(bytes);
        self
    }

    /// Include an inner error that caused this one.
    pub fn with_inner<E: Error + 'static>(mut self, inner: E) -> CodeError {
        self.inner = Some(Box::new(inner) as Box<Error>);
        self
    }

    /// Include an inner error that caused this one.
    pub fn set_inner(&mut self, inner: Option<Box<Error>>) {
        self.inner = inner;
    }

    pub fn with_name<T: Into<String>>(mut self, name: T) -> CodeError {
        self.encoding_name = Some(name.into());
        self
    }
}

impl fmt::Display for CodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "Encoding Error"));
        if let Some(ref name) = self.encoding_name {
            try!(write!(f, " in {}", name));
        }
        try!(write!(f, ": {}", self.message));
        if let Some(ref bytes) = self.bad_bytes {
            if bytes.is_empty() {
                try!(write!(f, " (input: [])"));
            } else {
                try!(write!(f, " (input: [{:02X}", bytes[0]));
                for byte in bytes.iter().skip(1) {
                    try!(write!(f, ",{:02X}", byte));
                }
                try!(write!(f, "])"));
            }
        }
        if let Some(ref e) = self.inner {
            try!(write!(f, "\ndue to {}", e));
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

/// Static methods relating to an encoding scheme.
pub trait EncodingStatics {
    /// Make a new instance of the Encoding, with the given options.
    /// Returns the encoding, or an error message if the options given are invalid.
    fn new(options: &str) -> Result<Box<Encoding>, String>;

    /// Print some descriptive text about the encoding, including the possible options that can be
    /// given when instantiating it.
    fn print_help();
}

pub type ByteIterator = Box<Iterator<Item = Result<u8, CodeError>>>;

/// An encoding scheme.
pub trait Encoding {
    /// Read from an input and produce the next output, or error.
    fn next(&mut self, input: &mut EncodingInput) -> Option<Result<Vec<u8>, CodeError>>;
}

/// An input that can yield single or multiple bytes.
pub trait EncodingInput {
    /// Get a single byte from the input.
    fn get_byte(&mut self) -> Option<Result<u8, CodeError>>;

    /// Get a specific number of bytes from the input.
    fn get_bytes(&mut self, n: usize) -> Option<Result<Vec<u8>, CodeError>>;

    /// Put a previously retrieved byte back in the input buffer.
    /// It will be the next one retrieved.
    fn unget_byte(&mut self, byte: u8);
}

// Internal structure for buffering the input to the Encoding. This is kept separate from Encoder
// so that it can be borrowed mutably while the Encoding is also borrowed mutably.
struct BufferedInput {
    pub input: ByteIterator,
    pub input_buffer: VecDeque<u8>,
}

impl BufferedInput {
    pub fn new(input: ByteIterator) -> BufferedInput {
        BufferedInput {
            input: input,
            input_buffer: VecDeque::new(),
        }
    }
}

/// An encoder. Takes an input, and an encoding, and produces output as needed.
/// The encodings can work on multiple bytes, but Encoder presents itself as a byte-oriented
/// iterator by buffering the data internally.
pub struct Encoder {
    encoding: Box<Encoding>,
    encoding_name: String,
    input: BufferedInput,
    output_buffer: VecDeque<u8>,
    stashed_error: Option<CodeError>,
}

impl Encoder {
    /// Make a new encoder, using the given byte-oriented iterator as input, and the given
    /// encoding.
    pub fn new<T: Into<String>>(input: ByteIterator, encoding: Box<Encoding>, enc_name: T)
            -> Encoder {
        Encoder {
            encoding: encoding,
            encoding_name: enc_name.into(),
            input: BufferedInput::new(input),
            output_buffer: VecDeque::new(),
            stashed_error: None,
        }
    }
}

impl Iterator for Encoder {
    type Item = Result<u8, CodeError>;
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        if self.output_buffer.is_empty() {
            match self.encoding.next(&mut self.input as &mut EncodingInput) {
                Some(Ok(bytes)) => {
                    self.output_buffer.extend(bytes);
                },
                Some(Err(e)) => {
                    debug!("{} returned error; stashing: {}", self.encoding_name, e);
                    // TODO: what if stashed_error is Some already?
                    self.stashed_error = Some(e.with_name(self.encoding_name.as_str()));
                },
                None => {
                    debug!("{} returned EOF", self.encoding_name);
                },
            }
        }

        if let Some(byte) = self.output_buffer.pop_front() {
            return Some(Ok(byte));
        }

        if let Some(err) = self.stashed_error.take() {
            debug!("{} returning stashed error", self.encoding_name);
            return Some(Err(err));
        }

        None
    }
}

impl EncodingInput for BufferedInput {
    fn get_byte(&mut self) -> Option<Result<u8, CodeError>> {
        if let Some(byte) = self.input_buffer.pop_front() {
            Some(Ok(byte))
        } else {
            self.input.next()
        }
    }

    fn get_bytes(&mut self, n: usize) -> Option<Result<Vec<u8>, CodeError>> {
        let mut result: Vec<u8> = Vec::with_capacity(n);
        while result.len() < n {
            if let Some(byte) = self.input_buffer.pop_front() {
                result.push(byte);
            } else {
                match self.input.next() {
                    Some(Ok(byte)) => { result.push(byte); },
                    Some(Err(e)) => {
                        error!("Error in adapter read: {}", e);
                        return Some(Err(CodeError::new(format!("error getting {} bytes", n))
                                                  .with_bytes(result)
                                                  .with_inner(e)));
                    },
                    None => {
                        if result.len() == 0 {
                            return None;
                        } else {
                            error!("premature EOF in BufferedInput: wanted {} bytes, only got {}",
                                   n, result.len());
                            return Some(Err(CodeError::new("premature EOF in input adapter")
                                                      .with_bytes(result)));
                        }
                    }
                }
            }
        }
        Some(Ok(result))
    }

    fn unget_byte(&mut self, byte: u8) {
        self.input_buffer.push_back(byte);
    }
}
