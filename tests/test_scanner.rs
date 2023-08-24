mod utils;

#[cfg(test)]
mod test_scanner {
    use crate::reg_token;
    use crsh::scanner::Scanner;
    use crsh::token::Token;

    #[test]
    fn scan_simple() {
        let command = "ls -a -b\n";
        let expected = vec![
            reg_token!("ls"),
            reg_token!("-a"),
            reg_token!("-b"),
            Token::EOF,
        ];
        let scanner = Scanner::new(command.into());
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(expected[..], tokens[..]);
    }

    #[test]
    fn scan_pipeline() {
        let command = "cat myfile | grep -r | wc\n";
        let expected = vec![
            reg_token!("cat"),
            reg_token!("myfile"),
            Token::Pipe,
            reg_token!("grep"),
            reg_token!("-r"),
            Token::Pipe,
            reg_token!("wc"),
            Token::EOF,
        ];
        let scanner = Scanner::new(command.into());
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(expected[..], tokens[..]);
    }

    #[test]
    fn scan_redirect() {
        let command = "grep hi < input >output";
        let expected = vec![
            reg_token!("grep"),
            reg_token!("hi"),
            Token::LRedirect,
            reg_token!("input"),
            Token::RRedirect,
            reg_token!("output"),
            Token::EOF,
        ];
        let scanner = Scanner::new(command.into());
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(expected[..], tokens[..]);
    }

    #[test]
    fn scan_quoted() {
        let command = "echo \"hi!     <\n\tthere&/;\"; cat 'my bad file name'";
        let expected = vec![
            reg_token!("echo"),
            reg_token!("hi!     <\n\tthere&/;"),
            Token::CommandSeparator,
            reg_token!("cat"),
            reg_token!("my bad file name"),
            Token::EOF,
        ];
        let scanner = Scanner::new(command.into());
        let tokens = scanner.scan_tokens().unwrap();
        assert_eq!(expected[..], tokens[..]);
    }
}
