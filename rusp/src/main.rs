#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

fn main() {
    let mut rl = Editor::<()>::new();
    loop {
        let readline = rl.readline("lispy >> ");
        match readline {
            Ok(line) => {
                if line == "exit" || line == "quit" {
                    break
                }
                rl.add_history_entry(&line);
                println!("{}", line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
}

