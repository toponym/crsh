use crsh::parser::Parser;
use crsh::scanner::Scanner;
use crsh::Crsh;
use std::io::{stdin, stdout, Write};
use std::process::exit;

fn main() {
    let mut interpreter = Crsh::new();
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
        // handle CTRL-D
        if input.len() == 0 {
            input = "exit".to_string();
        }
        // Eval
        let scanner = Scanner::new(input);
        let tokens = scanner.scan_tokens();
        let parser = Parser::new(tokens);
        let ast = parser.parse();
        match interpreter.execute(ast) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Error: {}", err)
            }
        }
    }
}
