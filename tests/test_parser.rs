mod utils;

#[cfg(test)]
mod tests {
    use crsh::parser::Parser;
    use crsh::ast::{Node, RedirectType};
    use crsh::token::Token;
    use crate::{reg_token, string_vec};
    
    #[test]
    fn parse_simple() {
        // "ls -a -b"
        let command = vec!(reg_token!("ls"), reg_token!("-a"), reg_token!("-b"), Token::EOF);
        let expected = Node::Pipeline(vec!(Node::Command(string_vec!("ls", "-a", "-b"), vec!())));
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
        let cmd0 = Node::Command(string_vec!("cat", "myfile"), vec!());
        let cmd1 = Node::Command(string_vec!("grep", "-r"), vec!());
        let cmd2 = Node::Command(string_vec!("wc"), vec!());
        let expected = Node::Pipeline(vec!(cmd0, cmd1, cmd2));
        let parser = Parser::new(command);
        assert_eq!(expected, parser.parse());
    }

    #[test]
    fn parse_redirect() {
        let command = vec!(reg_token!("grep"), reg_token!("hi"), Token::LRedirect,
        reg_token!("input"), Token::RRedirect, reg_token!("output"), Token::EOF);
        let redirect_vec = vec!(Node::Redirect(RedirectType::Read, "input".into()),
        Node::Redirect(RedirectType::Write, "output".into())
        );
        let expected = Node::Command(string_vec!("grep", "hi"), redirect_vec);
        let parser = Parser::new(command);
        assert_eq!(expected, parser.parse());
    }
}