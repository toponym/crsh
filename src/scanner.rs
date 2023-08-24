use crate::token::Token;
use lazy_static::lazy_static;
use std::collections::HashSet;

lazy_static! {
    static ref SPECIAL_CHARACTERS: HashSet<char> = "$'\"\\#=[]!><|;{}()*?~&".chars().collect();
}
#[derive(Debug)]
pub enum ScanError {
    EmptyToken(&'static str),
    IndexOutOfBounds(&'static str),
}
pub struct Scanner {
    chars: Vec<char>,
    curr: usize,
    tokens: Vec<Token>,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            chars: source.chars().collect(),
            curr: 0,
            tokens: vec![],
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, ScanError> {
        while !self.is_end() {
            let token_opt = self.scan_token()?;
            if let Some(token) = token_opt {
                self.tokens.push(token);
            }
        }
        self.tokens.push(Token::EOF);
        Ok(self.tokens)
    }

    fn scan_token(&mut self) -> Result<Option<Token>, ScanError> {
        macro_rules! advance_return {
            ($x:expr) => {{
                self.advance()?;
                return Ok(Some($x));
            }};
        }

        let chr = self.peek()?;
        match chr {
            '|' => advance_return!(Token::Pipe),
            ' ' | '\t' | '\n' | '\r' => self.whitespace(),
            '<' => advance_return!(Token::LRedirect),
            '>' => advance_return!(Token::RRedirect),
            ';' => advance_return!(Token::CommandSeparator),
            '"' | '\'' => self.quoted_token(),
            _ => self.regular_token(),
        }
    }

    fn quoted_token(&mut self) -> Result<Option<Token>, ScanError> {
        let mut token = String::new();
        let quote = *self.advance()?;
        while !(self.is_end() || *self.peek()? == quote) {
            token.push(*self.advance()?);
        }
        self.advance()?;
        Ok(Some(Token::Regular(token)))
    }

    fn regular_token(&mut self) -> Result<Option<Token>, ScanError> {
        let mut token = String::new();
        while !(self.is_end()
            || SPECIAL_CHARACTERS.contains(self.peek()?)
            || self.peek()?.is_whitespace())
        {
            token.push(*self.advance()?);
        }
        if token.is_empty() {
            return Err(ScanError::EmptyToken(
                "Regular token is empty. Current character: {:?}",
            ));
        }
        Ok(Some(Token::Regular(token)))
    }

    fn whitespace(&mut self) -> Result<Option<Token>, ScanError> {
        self.advance()?;
        Ok(None)
    }

    fn peek(&self) -> Result<&char, ScanError> {
        if self.is_end() {
            return Err(ScanError::IndexOutOfBounds("peek out of bounds"));
        }
        Ok(&self.chars[self.curr])
    }

    fn is_end(&self) -> bool {
        self.curr >= self.chars.len()
    }

    fn advance(&mut self) -> Result<&char, ScanError> {
        if self.is_end() {
            return Err(ScanError::IndexOutOfBounds("peek out of bounds"));
        }
        let curr_char = &self.chars[self.curr];
        self.curr += 1;
        Ok(curr_char)
    }
}
