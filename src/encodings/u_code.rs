use super::super::encoding::*;

use std::char;
use std::collections::VecDeque;
use std::mem;

pub struct UnUCode {
    input: InputBox,
    output_buffer: VecDeque<u8>,
}

impl CodeStatics for UnUCode {
    fn new(input: InputBox, _options: &str) -> Result<InputBox, String> {
        Ok(Box::new(UnUCode {
            input: input,
            output_buffer: VecDeque::new(),
        }))
    }

    fn print_help() {
        println!("Parses whitespace-separated 'U+XXXX' sequences into character data (UTF-32BE)");
        println!("(no options)");
    }
}

#[derive(Debug, PartialEq)]
enum State {
    U,
    Plus,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
}

fn next_state(state: State) -> State {
    match state {
        State::U => State::Plus,
        State::Plus => State::Digit1,
        State::Digit1 => State::Digit2,
        State::Digit2 => State::Digit3,
        State::Digit3 => State::Digit4,
        State::Digit4 => State::Digit5,
        State::Digit5 => State::Digit6,
        State::Digit6 => State::Digit7,
        State::Digit7 => State::Digit8,
        State::Digit8 => panic!("can't call next_state on Digit8"),
    }
}

fn hex_digit_value(c: u8) -> Option<u8> {
    if c >= b'0' && c <= b'9' {
        Some(c - b'0')
    } else if c >= b'a' && c <= b'f' {
        Some(c - b'a' + 10)
    } else if c >= b'A' && c <= b'F' {
        Some(c - b'A' + 10)
    } else {
        None
    }
}

impl UnUCode {
    fn parse_input(&mut self) -> Result<(), CodeError> {
        let mut codepoint = 0u32;
        let mut state = State::U;
        let mut bytes = vec![];

        loop {
            match self.input.next() {
                Some(Ok(byte)) => {
                    bytes.push(byte);

                    if byte == b' ' || byte == b'\t' || byte == b'\r' || byte == b'\n' {
                        match state {
                            State::U => (), // Ignore leading whitespace
                            State::Digit5 | State::Digit6 | State::Digit7 | State::Digit8 => {
                                // Whitespace after the first 4 digits ends the sequence
                                state = State::U;
                                break;
                            },
                            _ => {
                                error!("unexpected whitespace in state {:?}", state);
                                return Err(CodeError::new("unexpected whitespace")
                                                     .with_bytes(bytes));
                            }
                        }
                    } else {
                        let mut error = false;
                        match state {
                            State::U => {
                                if byte != b'U' {
                                    error = true;
                                }
                            },
                            State::Plus => {
                                if byte != b'+' {
                                    error = true;
                                }
                            },
                            State::Digit1 | State::Digit2 | State::Digit3 | State::Digit4 => {
                                if let Some(value) = hex_digit_value(byte) {
                                    let shift = match state {
                                        State::Digit1 => 12,
                                        State::Digit2 => 8,
                                        State::Digit3 => 4,
                                        State::Digit4 => 0,
                                        _ => unreachable!(),
                                    };
                                    codepoint |= (value as u32) << shift;
                                } else {
                                    error = true;
                                }
                            },
                            State::Digit5 | State::Digit6 | State::Digit7 | State::Digit8 => {
                                // whitespace was checked above
                                if let Some(value) = hex_digit_value(byte) {
                                    codepoint <<= 4;
                                    codepoint |= value as u32;
                                } else {
                                    error = true;
                                }
                            }
                        }

                        if error {
                            error!("unexpected {:?} in state {:?}",
                                unsafe { char::from_u32_unchecked(byte as u32) }, state);
                            return Err(CodeError::new("unexpected input while parsing U+ code")
                                                 .with_bytes(bytes));
                        } else {
                            if state == State::Digit8 {
                                state = State::U;
                                break;
                            } else {
                                state = next_state(state);
                            }
                        }
                    }
                },
                None => {
                    match state {
                        State::U => {
                            return Ok(());
                        }
                        State::Digit5 | State::Digit6 | State::Digit7 | State::Digit8 => {
                            state = State::U;
                            break;
                        },
                        _ => {
                            error!("unexpected EOF in state {:?}", state);
                            return Err(CodeError::new(format!("unexpected EOF in state {:?}", state))
                                                 .with_bytes(bytes));
                        }
                    }
                },
                Some(Err(e)) => {
                    return Err(CodeError::new(format!("error parsing U+XXXX sequence in state {:?}", state))
                                         .with_bytes(bytes)
                                         .with_inner(e));
                }
            }
        }

        // have codepoint, now push to output buffer.
        for i in 0..4 {
            let shift = 8 * (3 - i);
            self.output_buffer.push_back(((codepoint & (0xFF << shift)) >> shift) as u8);
        }

        Ok(())
    }
}

impl Code for UnUCode {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        if self.output_buffer.is_empty() {
            if let Err(e) = self.parse_input() {
                return Some(Err(e));
            }
        }

        if let Some(byte) = self.output_buffer.pop_front() {
            Some(Ok(byte))
        } else {
            None
        }
    }
}
