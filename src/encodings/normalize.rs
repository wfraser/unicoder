use super::super::encoding::*;
use super::utils;

use std::char;
use ucd::*;
use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char as unicode_char;

pub struct Normalize {
    form: NormalizationForm,
}

#[derive(Debug, PartialEq)]
enum NormalizationForm {
    NFD,
    NFKD,
    NFC,
    NFKC,
}

impl EncodingStatics for Normalize {
    fn new(options: &str) -> Result<Box<dyn Encoding>, String> {
        let form = match options {
            "nfd" => NormalizationForm::NFD,
            "nfkd" => NormalizationForm::NFKD,
            "nfc" => NormalizationForm::NFC,
            "nfkc" => NormalizationForm::NFKC,
            _ => {
                return Err(format!("invalid normalization form {:?}", options));
            }
        };

        Ok(Box::new(Normalize { form }))
    }

    fn print_help() {
        println!("Formats Unicode input into one of the Unicode Normalization Forms.");
        println!("Options:");
        println!("  nfc     Normalization Form C - Canonical Composition");
        println!("  nfkc    Normalization Form KC - Compatibility Composition");
        println!("  nfd     Normalization Form D - Canonical Decomposition");
        println!("  nfkd    Normalization Form KD - Compatibility Decomposition");
    }
}

impl Normalize {
    fn is_stable(&self, c: char) -> bool {
        unicode_char::canonical_combining_class(c) == 0 && match self.form {
            NormalizationForm::NFC => c.quick_check_nfc() == Trilean::True,
            NormalizationForm::NFKC => c.quick_check_nfkc() == Trilean::True,
            NormalizationForm::NFD => c.quick_check_nfd(),
            NormalizationForm::NFKD => c.quick_check_nfkd(),
        }
    }
}

fn codepoints_string(slice: &[char]) -> String {
    slice.iter().map(|c| format!("U+{:04X}", *c as u32)).collect::<Vec<_>>().join(", ")
}

impl Encoding for Normalize {
    fn next(&mut self, input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        let mut input_buffer = Vec::<u8>::new();
        let mut chars = Vec::<char>::new();
        loop {
            let bytes = match input.get_bytes(4) {
                Some(Ok(bytes)) => bytes,
                Some(Err(e)) => {
                    return Some(Err(CodeError::new("error reading character")
                                              .with_bytes(input_buffer)
                                              .with_inner(e)));
                },
                None => {
                    if input_buffer.is_empty() {
                        return None;
                    } else {
                        // EOF: Process the characters we have.
                        break;
                    }
                }
            };

            let codepoint = utils::u32_from_bytes(&bytes, true);
            let c = unsafe { char::from_u32_unchecked(codepoint) };

            if self.is_stable(c) && !chars.is_empty() {
                // Found the next stable character. Push it back, stop reading, and start work.
                debug!("Read 2nd stable code point; beginning processing.");
                for byte in bytes {
                    input.unget_byte(byte);
                }
                break;
            }

            chars.push(c);

            input_buffer.extend_from_slice(&bytes);
        }

        let iterator = chars.iter().cloned();
        let normalized: Vec<char> = match self.form {
            NormalizationForm::NFC => iterator.nfc().collect(),
            NormalizationForm::NFKC => iterator.nfkc().collect(),
            NormalizationForm::NFD => iterator.nfd().collect(),
            NormalizationForm::NFKD => iterator.nfkd().collect(),
        };

        debug!("({}) -> ({})", codepoints_string(&chars), codepoints_string(&normalized));

        let mut out = Vec::new();
        for c in normalized {
            out.extend_from_slice(&utils::u32_to_bytes(c as u32, true));
        }

        Some(Ok(out))
    }
}
