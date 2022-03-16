use std::env;
use aritexpr::token::intring::IntRingTokenParser;
use aritexpr::token::TokenIterator;
use itertools::Itertools;

fn main() {
    let mut args= env::args();
    args.next().expect("What");
    let str = args.next().expect("No argument");
    let iter = TokenIterator::new(&str, IntRingTokenParser::new());
    let tokens_result: Result<Vec<_>, _> = iter.collect();
    match tokens_result {
        Ok(tokens) => println!("Tokens: {}", tokens.iter().map(|wp| &wp.token).format(" ")),
        Err(err) => {
            eprintln!("{}: {}", err.message, str);
            eprintln!("{:>1$}", "^", err.message.len() + err.position + 3);
        },
    };

}
