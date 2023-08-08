use std::mem::discriminant;
use lazy_static::lazy_static;
use crate::token::Token;
use crate::ast::Node;

lazy_static!{
    static ref REGULAR_TOKEN: Token = Token::Regular("".to_string());
}

pub struct Parser{
    tokens: Vec<Token>,
    curr: usize
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser{
            tokens,
            curr: 0
        }
    }

    pub fn parse(mut self) -> Node{
        self.pipeline()
    }

    fn pipeline(&mut self) -> Node {
        let mut commands = vec!(self.command());
        while self.match_tok(&Token::Pipe){
            commands.push(self.command());
        }
        Node::Pipeline(commands)
    }

    fn command(&mut self) -> Node {
        let mut command = vec!();
        while self.check_tok(&REGULAR_TOKEN){
            // TODO no-copy approach instead?
            let tok = self.advance();
            match tok {
                Token::Regular(string) => command.push(string.clone()),
                // TODO improve error handling/propagation
                _ => eprintln!("Error parsing command")
            }
        }
        todo!("Add redirect");
        //Node::Command(command, vec!())
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