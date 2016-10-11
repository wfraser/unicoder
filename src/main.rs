#[macro_use]
extern crate log;
extern crate ucd;

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

impl Code for StdinCode {
    fn next(&mut self) -> Option<Result<u8, CodeError>> {
        match self.input.next() {
            Some(result) => Some(Ok(result.unwrap())),  // todo: needs IO error handling
            None => None
        }
    }
}

struct DebugOutput {
    debug_output: bool
}

impl log::Log for DebugOutput {
    fn enabled(&self, metadata: &log::LogMetadata) -> bool {
        if !self.debug_output {
            metadata.level() == log::LogLevel::Warn
        } else {
            true
        }
    }

    fn log(&self, record: &log::LogRecord) {
        if self.debug_output {
            writeln!(io::stdout(), "{}: {}: {}", record.target(), record.level(), record.args()).unwrap();
        }
    }
}

impl DebugOutput {
    pub fn init(debug: bool) {
        log::set_logger(|max_log_level| {
            max_log_level.set(if debug {
                log::LogLevelFilter::Debug
            } else {
                log::LogLevelFilter::Warn
            });
            Box::new(DebugOutput { debug_output: debug })
        }).expect("failed to initialize logging");
    }
}

fn main() {
    let mut args: Vec<_> = env::args().collect();

    let mut debug = false;
    let mut verbose = false;
    let mut help = false;
    let mut list = false;

    if args.len() == 1 {
        help = true;
    }

    let mut num_option_args = 0;
    for (i, arg) in args.iter().skip(1).enumerate() {
        num_option_args = i;
        if arg.starts_with("-") {
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
            }
        } else {
            break;
        }
    }

    args = args.split_off(num_option_args);

    if debug || verbose {
        DebugOutput::init(debug);
    }

    if help {
        if args.len() > 1 {
            for encoding in args.iter().skip(1) {
                println!("{}:", encoding);
                if let Err(msg) = print_help(encoding) {
                    println!("{}", msg);
                }
                println!("");
            }
        } else {
            println!("usage: {} [options] <encoding[,option,...]>...]", args[0]);
            println!("       {} {{--help|-h}} [encoding]", args[0]);
            println!("       {} --list", args[0]);
            println!("options:");
            println!("      -d | --debug        enable stderr debug output logging");
            println!("      -v | --verbose      enable stderr error output logging");
        }
        process::exit(-1);
    }

    if list {
        println!("list of available encodings:\n");
        print_all_help();
        process::exit(-1);
    }

    let mut input: Box<Code> = Box::new(StdinCode { input: io::stdin().bytes() });
    for encoding in args.iter().skip(1) {
        debug!("encoding: {}", encoding);
        let parts: Vec<&str> = encoding.splitn(2, ",").collect();
        input = get_code(parts[0], input, parts.get(1).unwrap_or(&"")).unwrap();
    }

    loop {
        match input.next() {
            None => { break; },
            Some(Ok(byte)) => { io::stdout().write(&[byte]).unwrap(); },
            Some(Err(e)) => { panic!(format!("{:?}", e)); },
        }
    }
}
