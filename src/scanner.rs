use std::collections::HashSet;
use lazy_static::lazy_static;

lazy_static!{
    static ref SPECIAL_CHARACTERS: HashSet<char> = "$'\"\\#=[]!><|;{}()*?~&".chars().collect();
}

// ignore warnings for variants that aren't implemented yet
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum RedirectionType {
    Read,
    Write,
    Append
}

// ignore warnings for variants that aren't implemented yet
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Regular(String),
    Expansion,
    SingleQuote,
    DoubleQuote,
    Assignment,
    Redirection(RedirectionType),
    Pipe,
    CommandSeparator,
    SubshellStart,
    SubshellEnd,
    Home,
    Background
}

pub struct Scanner {
    chars: Vec<char>,
    curr: usize,
    tokens: Vec<Token>
}

impl Scanner {
    pub fn new<'a>(source: String) -> Self{
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

    
    fn peek<'a>(&'a self) -> &'a char {
        &self.chars[self.curr]
    }
    
    fn is_end(&self) -> bool{
        self.curr >= self.chars.len()
    }

    fn advance<'a>(&'a mut self) -> &'a char {
        let curr_char = &self.chars[self.curr];
        self.curr += 1;
        curr_char
    }
}