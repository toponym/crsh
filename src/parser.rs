use crate::ast::Node;
use crate::token::Token;
use lazy_static::lazy_static;
use std::mem::discriminant;

lazy_static! {
    static ref REGULAR_TOKEN: Token = Token::Regular("".to_string());
}

#[derive(Debug)]
pub enum ParseError {
    TokensNotParsed(&'static str),
    NotExpectedToken(&'static str),
    IndexOutOfBounds(&'static str),
}

macro_rules! unwrap_regular {
    ($x:expr) => {{
        if let Token::Regular(string) = $x {
            string
        } else {
            return Err(ParseError::NotExpectedToken("Token is not Regular Token"));
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

    pub fn parse(mut self) -> Result<Node, ParseError> {
        let mut pipelines = vec![self.pipeline()?];
        while self.match_tok(&Token::CommandSeparator)? {
            pipelines.push(self.pipeline()?);
        }
        if self.peek()? != &Token::EOF {
            return Err(ParseError::TokensNotParsed("Not all tokens are parsed"));
        }
        if pipelines.len() == 1 {
            Ok(pipelines.pop().unwrap())
        } else {
            Ok(Node::CommandSequence(pipelines))
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tokens == vec![Token::EOF]
    }

    fn pipeline(&mut self) -> Result<Node, ParseError> {
        let mut commands = vec![self.command()?];
        while self.match_tok(&Token::Pipe)? {
            commands.push(self.command()?);
        }
        Ok(Node::Pipeline(commands))
    }

    fn command(&mut self) -> Result<Node, ParseError> {
        let mut command = vec![];
        while self.check_tok(&REGULAR_TOKEN)? {
            let string = unwrap_regular!(self.advance());
            // TODO no-copy approach instead?
            command.push(string.clone());
        }
        let mut redirect = vec![];
        while self.check_tok(&Token::RRedirect)? || self.check_tok(&Token::LRedirect)? {
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
                    _ => return Err(ParseError::NotExpectedToken("Unexpected token after \">\"")),
                },
                _ => {
                    return Err(ParseError::NotExpectedToken(
                        "Unexpected token while attempting to match Redirect",
                    ))
                }
            }
        }
        Ok(Node::Command(command, redirect))
    }

    fn check_tok(&self, token: &Token) -> Result<bool, ParseError> {
        Ok(discriminant(self.peek()?) == discriminant(token))
    }

    fn match_tok(&mut self, token: &Token) -> Result<bool, ParseError> {
        let same_enum = self.check_tok(token)?;
        if same_enum {
            self.advance();
        }
        Ok(same_enum)
    }

    fn peek(&self) -> Result<&Token, ParseError> {
        if self.tokens.len() <= self.curr {
            return Err(ParseError::IndexOutOfBounds("peek out of bounds"));
        }
        Ok(&self.tokens[self.curr])
    }

    fn advance(&mut self) -> &Token {
        let curr_char = &self.tokens[self.curr];
        self.curr += 1;
        curr_char
    }
}
