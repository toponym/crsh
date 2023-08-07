mod utils;

#[cfg(test)]
mod tests {
    use crsh::parser::Parser;
    use crsh::ast::Node;
    use crsh::token::Token;
    use crate::{reg_token, string_vec};
    
    #[test]
    fn parse_simple() {
        // "ls -a -b"
        let command = vec!(reg_token!("ls"), reg_token!("-a"), reg_token!("-b"), Token::EOF);
        let expected = Node::Pipeline(vec!(Node::Command(string_vec!("ls", "-a", "-b"))));
        let parser = Parser::new(command);
        assert_eq!(expected, parser.parse());
    }
    
    #[test]
    fn parse_pipeline() {
        // "cat myfile | grep -r | wc"
        let command = vec!(reg_token!("cat"), reg_token!("myfile"), Token::Pipe, 
        reg_token!("grep"), reg_token!("-r"), Token::Pipe, reg_token!("wc"),
        Token::EOF
        );
        let cmd0 = Node::Command(string_vec!("cat", "myfile"));
        let cmd1 = Node::Command(string_vec!("grep", "-r"));
        let cmd2 = Node::Command(string_vec!("wc"));
        let expected = Node::Pipeline(vec!(cmd0, cmd1, cmd2));
        let parser = Parser::new(command);
        assert_eq!(expected, parser.parse());
    }
}