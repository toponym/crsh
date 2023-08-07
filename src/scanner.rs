use std::collections::HashSet;
use lazy_static::lazy_static;
use crate::token::Token;

lazy_static!{
    static ref SPECIAL_CHARACTERS: HashSet<char> = "$'\"\\#=[]!><|;{}()*?~&".chars().collect();
}

pub struct Scanner {
    chars: Vec<char>,
    curr: usize,
    tokens: Vec<Token>
}

impl Scanner {
    pub fn new(source: String) -> Self{
        Self{
            chars: source.chars().collect(),
            curr: 0,
            tokens: vec!()
        }
    }

    pub fn scan_tokens(mut self) -> Vec<Token>{
        while !self.is_end() {
            let token_opt = self.scan_token();
            if let Some(token) = token_opt{
                self.tokens.push(token);
            }
        }
        self.tokens.push(Token::EOF);
        self.tokens
    }

    fn regular_token(&mut self) -> Option<Token>{
        let mut token = String::new();
        while !(self.is_end() || SPECIAL_CHARACTERS.contains(self.peek()) || self.peek().is_whitespace()){
            token.push(*self.advance());
        }
        if token.is_empty(){
            // TODO better error handling
            panic!("Error: regular token is empty. Current character: {:?}", self.peek());
        }
        Some(Token::Regular(token))
    }

    fn whitespace(&mut self) -> Option<Token>{
        self.advance();
        None
    }

    fn pipe_token(&mut self) -> Option<Token>{
        self.advance();
        Some(Token::Pipe)
    }

    fn scan_token(&mut self) -> Option<Token>{
        let chr = self.peek();
        match chr {
            '|' => self.pipe_token(),
            ' '|'\t'|'\n'|'\r' => self.whitespace(),
            _ => self.regular_token()
        }
    }

    
    fn peek(& self) -> &char {
        &self.chars[self.curr]
    }
    
    fn is_end(&self) -> bool{
        self.curr >= self.chars.len()
    }

    fn advance(& mut self) -> & char {
        let curr_char = &self.chars[self.curr];
        self.curr += 1;
        curr_char
    }
}