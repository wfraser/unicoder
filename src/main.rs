#[macro_use]
extern crate log;
extern crate ucd;
extern crate unicode_names;

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
            metadata.level() <= log::LogLevel::Warn
        } else {
            true
        }
    }

    fn log(&self, record: &log::LogRecord) {
        if self.enabled(record.metadata()) {
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
    let mut args: VecDeque<_> = env::args().collect();

    let mut debug = false;
    let mut verbose = false;
    let mut help = false;
    let mut list = false;

    if args.len() == 1 {
        help = true;
    }

    let program_name = args.pop_front().unwrap();

    while !args.is_empty() {
        let arg = args.pop_front().unwrap();
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
                println!("");
            }
        } else {
            println!("usage: {} [options] <encoding[,option,...]>...]", program_name);
            println!("       {} {{--help|-h}} [encoding]", program_name);
            println!("       {} --list", program_name);
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
    for encoding in args {
        debug!("encoding: {}", encoding);
        let parts: Vec<&str> = encoding.splitn(2, ",").collect();
        input = get_code(parts[0], input, parts.get(1).unwrap_or(&"")).unwrap();
    }

    loop {
        match input.next() {
            None => { break; },
            Some(Ok(byte)) => { io::stdout().write(&[byte]).unwrap(); },
            Some(Err(e)) => {
                println!("\nError processing input:\n{}", e);
                println!("terminating.");
                process::exit(1);
            },
        }
    }
}
