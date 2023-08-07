mod utils;

#[cfg(test)]
mod test_scanner {
    use crsh::scanner::Scanner;
    use crsh::token::Token;
    use crate::reg_token;
    
    #[test]
    fn scan_simple() {
        let command = "ls -a -b\n";
        let expected = vec!(reg_token!("ls"), reg_token!("-a"), reg_token!("-b"), Token::EOF);
        let scanner = Scanner::new(command.into());
        let tokens = scanner.scan_tokens();
        assert_eq!(expected[..], tokens[..]);
    }

    #[test]
    fn scan_pipeline() {
        let command = "cat myfile | grep -r | wc\n";
        let expected = vec!(reg_token!("cat"), reg_token!("myfile"), Token::Pipe, 
            reg_token!("grep"), reg_token!("-r"), Token::Pipe, reg_token!("wc"),
            Token::EOF
        );
        let scanner = Scanner::new(command.into());
        let tokens = scanner.scan_tokens();
        assert_eq!(expected[..], tokens[..]);
    }
}