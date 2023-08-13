use crate::ast::Node;
use crate::token::Token;
use lazy_static::lazy_static;
use std::mem::discriminant;

lazy_static! {
    static ref REGULAR_TOKEN: Token = Token::Regular("".to_string());
}

macro_rules! unwrap_regular {
    ($x:expr) => {{
        if let Token::Regular(string) = $x {
            string
        } else {
            panic!("Expected Regular Token")
        }
    }};
}

pub struct Parser {
    tokens: Vec<Token>,
    curr: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, curr: 0 }
    }

    pub fn parse(mut self) -> Node {
        let mut pipelines = vec![self.pipeline()];
        while self.match_tok(&Token::CommandSeparator) {
            pipelines.push(self.pipeline());
        }
        // TODO gracefully handle parsing errors
        assert!(self.peek() == &Token::EOF, "Not all tokens are parsed");
        if pipelines.len() == 1 {
            pipelines.pop().unwrap()
        } else {
            Node::CommandSequence(pipelines)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tokens == vec!(Token::EOF)
    }

    fn pipeline(&mut self) -> Node {
        let mut commands = vec![self.command()];
        while self.match_tok(&Token::Pipe) {
            commands.push(self.command());
        }
        Node::Pipeline(commands)
    }

    fn command(&mut self) -> Node {
        // TODO improve error handling/propagation
        // should gracefully handle parsing errors, not panic
        let mut command = vec![];
        while self.check_tok(&REGULAR_TOKEN) {
            let string = unwrap_regular!(self.advance());
            // TODO no-copy approach instead?
            command.push(string.clone());
        }
        let mut redirect = vec![];
        while self.check_tok(&Token::RRedirect) || self.check_tok(&Token::LRedirect) {
            let tok = self.advance();
            match tok {
                Token::LRedirect => {
                    let string = unwrap_regular!(self.advance());
                    redirect.push(Node::RedirectRead(string.clone()))
                }
                Token::RRedirect => match self.advance() {
                    Token::RRedirect => {
                        let string = unwrap_regular!(self.advance());
                        redirect.push(Node::RedirectAppend(string.clone()));
                    }
                    Token::Regular(string) => {
                        redirect.push(Node::RedirectWrite(string.clone()));
                    }
                    tok => panic!("Unexpected token after \">\": {:?}", tok),
                },
                _ => panic!("Error parsing command"),
            }
        }
        Node::Command(command, redirect)
    }

    fn check_tok(&self, token: &Token) -> bool {
        discriminant(self.peek()) == discriminant(token)
    }

    fn match_tok(&mut self, token: &Token) -> bool {
        let same_enum = self.check_tok(token);
        if same_enum {
            self.advance();
        }
        same_enum
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.curr]
    }

    fn advance(&mut self) -> &Token {
        let curr_char = &self.tokens[self.curr];
        self.curr += 1;
        curr_char
    }
}
