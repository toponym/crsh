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
        if input.is_empty() {
            input = "exit".to_string();
        }
        // Eval
        let scanner = Scanner::new(input);
        let tokens = match scanner.scan_tokens() {
            Ok(scanner) => scanner,
            Err(msg) => {
                eprintln!("Scanning error: {:?}", msg);
                continue;
            }
        };
        let parser = Parser::new(tokens);
        if parser.is_empty() {
            continue;
        }
        let ast = match parser.parse() {
            Ok(parser) => parser,
            Err(msg) => {
                eprintln!("Parsing error: {:?}", msg);
                continue;
            }
        };
        match interpreter.execute(ast) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("Execution error: {}", err)
            }
        }
    }
}
