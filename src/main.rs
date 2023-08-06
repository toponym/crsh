use std::io::{stdin, stdout, Write};
use std::process::{exit};
use crsh::Crsh;
use crsh::scanner::Scanner;

fn main() {
    loop {
        print!("> ");
        stdout().flush().unwrap_or_else(|_| {
            eprintln!("Error flushing output");
            exit(1);
        });
        // Read
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Ok(_) => (),
            Err(_) => {
                eprintln!("Error reading input");
                exit(1);
            }
        }
        // Eval
        let scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens();
        let ast = Crsh::parse(tokens);
        match Crsh::execute(ast){
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error: {}", err)
            }
        }
    }
}
