
use std::iter::{Peekable, Enumerate};
use std::str::Chars;
use core::result;
use std::error;
use std::fmt::{Display, Formatter, Debug};
use std::hash::Hash;

pub mod intring;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TokenError {
    pub message: String,
    pub position: usize
}

impl Display for TokenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unparseable input at position {}: {}", self.position, self.message)
    }
}

impl error::Error for TokenError {
}

pub type TokenResult<T> = result::Result<T, TokenError>;

pub trait Token : Display + PartialEq + Eq + Hash + Clone {
}

/// Parses strings to tokens.
pub trait TokenParser {
    type TokenType: Token;

    /// Try to parse next token in char sequence from iterator.
    fn read_next_token<I: Iterator<Item=(usize, char)>>(
        &self,
        char_iterator: &mut Peekable<I>) -> TokenResult<Self::TokenType>;

}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct TokenWithPos<T: Token> {
    pub token: T,
    pub position: usize
}

/// A token iterator based on a string input and a [TokenParser]
pub struct TokenIterator<T: Token, I: Iterator<Item=(usize, char)>, G: TokenParser<TokenType=T>> {
    char_iterator: Peekable<I>,
    token_generator: G
}

impl<T: Token, G: TokenParser<TokenType=T>> TokenIterator<T, Enumerate<Chars<'_>>, G> {
    pub fn new(str: &impl AsRef<str>, token_generator: G) -> TokenIterator<T, Enumerate<Chars<'_>>, G> {
        TokenIterator {
            char_iterator: str.as_ref().chars().enumerate().peekable(),
            token_generator
        }
    }
}

impl<T: Token, I: Iterator<Item=(usize, char)>, G: TokenParser<TokenType=T>> Iterator
for TokenIterator<T, I, G> {
    type Item = TokenResult<TokenWithPos<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.char_iterator.next_if(|c| c.1.is_whitespace()).is_some() {}

        if self.char_iterator.peek().is_none() {
            return None
        }

        let position = self.char_iterator.peek().unwrap().0;
        Some(
            match self.token_generator.read_next_token(&mut self.char_iterator) {
                Ok(token) => Ok(TokenWithPos{token, position}),
                Err(err) => Err(err),
            }
        )
    }
}

// pub fn tokenize<G, T, R>(read: R, tokenizer: T) -> impl Iterator<Item=io::Result<T>>
//     where T: Token, G: TokenGenerator<T>, R: BufRead {
//     read.has_data_left()
//     i64::from_str()
// }

