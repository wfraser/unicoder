#![allow(clippy::new_ret_no_self)]
#![deny(rust_2018_idioms)]

#[macro_use]
extern crate log;

use std::collections::VecDeque;
use std::env;
use std::io::{self, Read, Write};
use std::process;

mod encoding;
use encoding::*;

mod encodings;
use encodings::*;

struct StdinCode {
    input: io::Bytes<io::Stdin>,
}

impl Iterator for StdinCode {
    type Item = Result<u8, CodeError>;
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        self.input.next().map(|result|
            result.map_err(|ioerr|
                CodeError::new("I/O error in stdin").with_inner(ioerr)))
    }
}

struct IdentityEncoding;

impl Encoding for IdentityEncoding {
    fn next(&mut self, input: &mut dyn EncodingInput) -> Option<Result<Vec<u8>, CodeError>> {
        input.get_bytes(1)
    }
}

struct DebugOutput {
    debug_output: bool
}

impl log::Log for DebugOutput {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        if !self.debug_output {
            metadata.level() <= log::Level::Warn
        } else {
            true
        }
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            println!("{}: {}: {}", record.target(), record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

impl DebugOutput {
    pub fn init(debug: bool) {
        log::set_max_level(if debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Warn
        });
        log::set_boxed_logger(Box::new(DebugOutput { debug_output: debug }))
            .expect("failed to initialize logging");
    }
}

pub fn parse_error_policy(s: &str) -> Option<ErrorPolicy> {
    if s == "halt" {
        Some(ErrorPolicy::Halt)
    } else if s == "skip" {
        Some(ErrorPolicy::Skip)
    } else if s == "replace" {
        Some(ErrorPolicy::Replace)
    } else {
        None
    }
}

fn main() {
    let mut args: VecDeque<_> = env::args().collect();

    let mut debug = false;
    let mut verbose = false;
    let mut help = false;
    let mut list = false;

    if args.len() == 1 {
        help = true;
    }

    let program_name = args.pop_front().unwrap();

    let mut error_policy = ErrorPolicy::Halt;

    while !args.is_empty() {
        let arg = args.pop_front().unwrap();
        if arg.starts_with('-') {
            if !arg.starts_with("--") {
                for c in arg.chars().skip(1) {
                    match c {
                        'd' => { debug = true; },
                        'v' => { verbose = true; },
                        'h' => { help = true; },
                        _ => {
                            println!("unknown flag {:?}", c);
                            process::exit(-1);
                        }
                    }
                }
            } else if arg == "--debug" {
                debug = true;
            } else if arg == "--verbose" {
                verbose = true;
            } else if arg == "--list" {
                list = true;
            } else if arg == "--help" {
                help = true;
            } else if let Some(s) = arg.strip_prefix("--errors=") {
                error_policy = parse_error_policy(s).unwrap_or_else(|| {
                    println!("invalid error policy");
                    process::exit(-1);
                });
            } else {
                println!("unknown option {:?}", arg);
                process::exit(-1);
            }
        } else {
            // put the argument back
            args.push_front(arg);
            break;
        }
    }

    if debug || verbose {
        DebugOutput::init(debug);
    }

    if help {
        if !args.is_empty() {
            for ref encoding in args {
                println!("{}:", encoding);
                if let Err(msg) = print_help(encoding) {
                    println!("{}", msg);
                }
                println!();
            }
        } else {
            println!("usage: {} [options] <encoding[,option,...]>...]", program_name);
            println!("       {} {{--help|-h}} [encoding]", program_name);
            println!("       {} --list", program_name);
            println!("options:");
            println!("      -d | --debug        enable stderr debug output logging");
            println!("      -v | --verbose      enable stderr error output logging");
            println!("      --errors=<error policy>");
            println!("          halt            exit on errors (default)");
            println!("          skip            skip over erroneous input");
            println!("          replace         substitute with the encoding's replacement");
            println!("                            character (e.g. U+FFFD or '?')");
        }
        process::exit(-1);
    }

    if list {
        println!("list of available encodings:\n");
        print_all_help();
        process::exit(-1);
    }

    debug!("using error policy {:?}", error_policy);

    let stdin = Box::new(StdinCode { input: io::stdin().bytes() });
    let mut encoder: Box<Encoder> = Box::new(Encoder::new(stdin, Box::new(IdentityEncoding), "stdin", error_policy));
    for encoding_name in args {
        debug!("encoding: {}", encoding_name);
        let parts: Vec<&str> = encoding_name.splitn(2, ',').collect();
        let encoding = match get_encoding(parts[0], parts.get(1).unwrap_or(&"")) {
            Ok(enc) => enc,
            Err(msg) => {
                println!("Error setting up {}: {}", parts[0], msg);
                process::exit(-1);
            }
        };
        encoder = Box::new(Encoder::new(encoder, encoding, encoding_name.as_str(), error_policy));
    }

    loop {
        match encoder.next() {
            None => { break; },
            Some(Ok(byte)) => { io::stdout().write_all(&[byte]).unwrap(); },
            Some(Err(e)) => {
                println!("\nError processing input:\n{}", e);
                println!("terminating.");
                process::exit(1);
            },
        }
    }
}
