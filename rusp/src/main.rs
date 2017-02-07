use std::io;
use std::io::prelude::*;

fn main() {    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    loop {
        write!(&mut stdout, "lispy > ").expect("Could not write to stdout");
        stdout.flush().expect("Could not flush stdout");

        let mut input = String::new();
        stdin.read_line(&mut input).expect("Could not read line from stdin");

        input.pop(); // pop trailing newline
        if input == "exit" {
            break;
        }

        writeln!(&mut stdout, "Input: {}", input).expect("Could not write line to stdout");
    }
}
