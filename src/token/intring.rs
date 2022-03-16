use crate::token::{Token, TokenParser, TokenResult, TokenError};
use std::iter::Peekable;
use crate::token::intring::IntRingToken::{LeftParenthesis, MultiplicationSign, MinusSign, PlusSign, RightParenthesis, DecimalInteger, Modulo, DivisionSign};
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum IntRingToken {
    LeftParenthesis,
    RightParenthesis,
    PlusSign,
    MinusSign,
    MultiplicationSign,
    DivisionSign,
    DecimalInteger(i64),
    Modulo
}

impl Display for IntRingToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IntRingToken::LeftParenthesis => f.write_char('(')?,
            IntRingToken::RightParenthesis => f.write_char(')')?,
            IntRingToken::PlusSign => f.write_char('+')?,
            IntRingToken::MinusSign => f.write_char('-')?,
            IntRingToken::MultiplicationSign => f.write_char('*')?,
            IntRingToken::DivisionSign => f.write_char('/')?,
            IntRingToken::DecimalInteger(d) => write!(f, "{}", d)?,
            IntRingToken::Modulo => f.write_str("mod")?,
        };
        Ok(())
    }
}

impl Token for IntRingToken {

}

pub struct IntRingTokenParser {
}

impl IntRingTokenParser {
    pub fn new() -> IntRingTokenParser {
        IntRingTokenParser{}
    }
}

impl TokenParser for IntRingTokenParser {
    type TokenType = IntRingToken;

    fn read_next_token<I: Iterator<Item=(usize, char)>>(
        &self, char_iterator: &mut Peekable<I>) -> TokenResult<Self::TokenType>
    {
        fn invalid_token_result(pos: usize) -> TokenResult<IntRingToken> {
            Err(TokenError{message: format!("Invalid token"), position: pos})
        }

        match char_iterator.peek().copied().unwrap() {
            (_, '(') => {char_iterator.next(); Ok(LeftParenthesis)},
            (_, ')') => {char_iterator.next(); Ok(RightParenthesis)},
            (_, '+') => {char_iterator.next(); Ok(PlusSign)},
            (_, '-') => {char_iterator.next(); Ok(MinusSign)},
            (_, '*') => {char_iterator.next(); Ok(MultiplicationSign)},
            (_, '/') => {char_iterator.next(); Ok(DivisionSign)},
            (pos, 'm') => {
                let str: String = char_iterator.take(3).map(|(_, c)| c).collect();
                if str == "mod" {
                    Ok(Modulo)
                } else {
                    invalid_token_result(pos)
                }

            },
            (pos, c) if c.is_numeric() => {
                let mut decimals = String::new();
                // while let Some(&c @ '0'..='9') = char_iterator.peek() {
                //     char_iterator.next();
                //     decimal.push(c);
                // }
                while let Some((_, c)) = char_iterator.next_if(|(_, c)| c.is_numeric()) {
                    decimals.push(c);
                }
                let parse_result = decimals.parse();
                match parse_result {
                    Ok(d) => Ok(DecimalInteger(d)),
                    Err(_) => Err(TokenError{message: "Decimal number too big".to_string(), position: pos}),
                }
            }
            (pos, _) => invalid_token_result(pos)
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::token::{TokenIterator, TokenWithPos};
    use crate::token::intring::IntRingTokenParser;
    use crate::token::intring::IntRingToken::{LeftParenthesis, RightParenthesis, PlusSign, MinusSign, MultiplicationSign, DecimalInteger, Modulo, DivisionSign};

    #[test]
    fn parse_single_token() {
        let str = "(";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: LeftParenthesis, position: 0})), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn parse_string() {
        let str = "(".to_string();
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: LeftParenthesis, position: 0})), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn parse_two_tokens() {
        let str = "((";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: LeftParenthesis, position: 0})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: LeftParenthesis, position: 1})), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn parse_with_whitespace() {
        let str = "  (  (  ";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: LeftParenthesis, position: 2})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: LeftParenthesis, position: 5})), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn parse_parentheses_and_operators() {
        let str = "()+-*/";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: LeftParenthesis, position: 0})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: RightParenthesis, position: 1})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: PlusSign, position: 2})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: MinusSign, position: 3})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: MultiplicationSign, position: 4})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: DivisionSign, position: 5})), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn parse_modulo() {
        let str = "5 mod 7";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: DecimalInteger(5), position: 0})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: Modulo, position: 2})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: DecimalInteger(7), position: 6})), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn invalid_token_starting_with_m() {
        let str = "5 mm";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: DecimalInteger(5), position: 0})), iter.next());
        let token_result = iter.next().unwrap();
        let err = token_result.expect_err("should be error");
        assert_eq!(2, err.position);
        assert_eq!("Invalid token", err.message);
    }

    #[test]
    fn parse_int_token() {
        let str = "1234567890";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: DecimalInteger(1234567890), position: 0})), iter.next());
        assert_eq!(None, iter.next());

        let str = "91";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: DecimalInteger(91), position: 0})), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn parse_int_token_other_tokens_before_and_after() {
        let str = "(12)";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: LeftParenthesis, position: 0})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: DecimalInteger(12), position: 1})), iter.next());
        assert_eq!(Some(Ok(TokenWithPos{token: RightParenthesis, position: 3})), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn parse_int_token_whitespace_before_and_after() {
        let str = "  12  ";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        assert_eq!(Some(Ok(TokenWithPos{token: DecimalInteger(12), position: 2})), iter.next());
        assert_eq!(None, iter.next());
    }

    #[test]
    fn parse_int_token_too_big() {
        let str = "()12312312312312123123123123123";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        iter.next().unwrap().unwrap();
        iter.next().unwrap().unwrap();
        let token_result = iter.next().unwrap();
        let err = token_result.expect_err("should be error");
        assert_eq!(2, err.position);
        assert_eq!("Decimal number too big", err.message);

    }

    #[test]
    fn chars_not_token() {
        let str = "() hest 2";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        iter.next().unwrap().unwrap();
        iter.next().unwrap().unwrap();
        let token_result = iter.next().unwrap();
        let err = token_result.expect_err("should be error");
        assert_eq!(3, err.position);
        assert_eq!("Invalid token", err.message);
    }

    #[test]
    fn display() {
        let str = "()+-*/123mod";
        let mut iter = TokenIterator::new(&str, IntRingTokenParser::new());

        while let Some(token_result) = iter.next() {
            println!("{}", token_result.unwrap().token);
        }
    }
}