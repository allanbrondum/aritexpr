use std::env;
use aritexpr::expression::parser::parse_int_ring_expression;

fn main() {
    let mut args= env::args();
    args.next().expect("What");
    let str = args.next().expect("No argument");
    match parse_int_ring_expression(&str) {
        Ok(expr) => {
            match expr.evaluate() {
                Ok(element) => {
                    println!("Result: {}" , element);
                },
                Err(err) => {
                    eprintln!("{}: {}", err.message, str);
                    // eprintln!("{:>1$}", "^", err.message.len() + err.p.);
                },
            };
        },
        Err(err) => {
            eprintln!("{}: {}", err.message, str);
            eprintln!("{:>1$}", "^", err.message.len() + err.position + 3);
        },
    };

}
