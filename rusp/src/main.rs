use std::io::{self, BufRead};

fn main() {
    loop {
        let mut line = String::new();

        io::stdin()
            .read_line(&mut line)
            .expect("failed to read line");
        println!("{}", line);
    }
}
