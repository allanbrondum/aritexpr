use crate::token::{TokenIterator, TokenError, TokenResult, TokenWithPos};
use crate::token::intring::{IntRingTokenParser, IntRingToken};
use crate::expression::ExpressionComponent;
use crate::expression::ring::intring::{IntRing};
use core::fmt;
use std::fmt::Formatter;
use std::{error, result};
use crate::expression::parser::ParseExpressionErrorKind::{TokenParseError, Unspecified, NoExpression};
use std::mem::swap;
use std::iter::Peekable;
use std::fs::set_permissions;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct ParseExpressionError {
    pub message: String,
    pub position: usize,
    pub kind: ParseExpressionErrorKind,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ParseExpressionErrorKind {
    Unspecified,
    TokenParseError,
    NoExpression,
}

impl fmt::Display for ParseExpressionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing expression at position {}: {}", self.position, self.message)
    }
}

impl error::Error for ParseExpressionError {
}

impl From<TokenError> for ParseExpressionError {
    fn from(err: TokenError) -> Self {
        ParseExpressionError {
            message: err.message,
            position: err.position,
            kind: TokenParseError,
        }
    }
}

pub type ParseExpressionResult<T> = result::Result<T, ParseExpressionError>;

fn create_err<T>(format_args: fmt::Arguments, position: usize, kind: ParseExpressionErrorKind) -> ParseExpressionResult<T> {
    Err(ParseExpressionError{message: format_args.to_string(), position, kind})
}

pub fn parse_int_ring_expression(
    str: impl AsRef<str>)
    -> ParseExpressionResult<ExpressionComponent<IntRing>>
{
    let tokens_result: TokenResult<Vec<TokenWithPos<IntRingToken>>> =
        TokenIterator::new(&str, IntRingTokenParser::new()).collect();
    let tokens = tokens_result?;

    parse_int_ring_expression_from_tokens(tokens)
}

/// Parse expression from `tokens`
pub fn parse_int_ring_expression_from_tokens(
    tokens: Vec<TokenWithPos<IntRingToken>>)
    -> ParseExpressionResult<ExpressionComponent<IntRing>>
{
    // TODO try implement polish notation intermediate result, simpler?

    let mut parsed_expression: Option<ExpressionComponent<IntRing>> = None;
    let mut tokens_iter = tokens.iter().rev().peekable();
    let result = parse_int_ring_expression_from_tokens_rec
        (&mut tokens_iter, &mut parsed_expression, false);

    if let Ok(_) = result {
        debug_assert!(tokens_iter.next().is_none());
    }

    match result {
        Ok(Some(expr)) => Ok(expr),
        Err(err) => Err(err),
        Ok(None) => create_err(format_args!("No expression"), 0, NoExpression)
    }
}

/// Parse and consume `tokens` in order to parse an expression. The token iterator may start
/// inside an expression where a potential right hand side for an operator is already parsed
/// into `parsed_expression`. The iterator may also start inside a parenthesis in which
/// case `has_open_parenthesis` is `true`.
///
fn parse_int_ring_expression_from_tokens_rec<'a, I>(
    tokens: &mut Peekable<I>,
    parsed_expression: &mut Option<ExpressionComponent<IntRing>>,
    has_open_parenthesis: bool)
    -> ParseExpressionResult<Option<ExpressionComponent<IntRing>>>
    where I: Iterator<Item=&'a TokenWithPos<IntRingToken>>
{
    let token_option = tokens.peek();

    if token_option.is_none() {
        if let Some(expr) = parsed_expression.take() {
            return Ok(Some(expr));
        } else {
            return Ok(None);
        }
    }

    let position = token_option.unwrap().position;
    let token = &token_option.unwrap().token;

    match &token {
        IntRingToken::DecimalInteger(d) => {
            tokens.next();
            if let Some(_) = parsed_expression.replace(ExpressionComponent::new_int_element(*d)) {
                return create_err(format_args!("Ring element cannot be followed by another ring element in expression"), position, Unspecified);
            }
            let rest = parse_int_ring_expression_from_tokens_rec(tokens, parsed_expression, has_open_parenthesis)?;
            if let Some(_) = rest {
                debug_assert!(parsed_expression.is_none());
                Ok(rest)
            } else {
                Ok(Some(parsed_expression.take().unwrap()))
            }
        },
        operator @ (IntRingToken::PlusSign | IntRingToken::MinusSign | IntRingToken::MultiplicationSign | IntRingToken::DivisionSign) => {
            tokens.next();
            let construct_expression = match operator {
                IntRingToken::PlusSign => ExpressionComponent::new_addition,
                IntRingToken::MinusSign => ExpressionComponent::new_subtraction,
                IntRingToken::MultiplicationSign => ExpressionComponent::new_multiplication,
                IntRingToken::DivisionSign => ExpressionComponent::new_division,
                _ => panic!("Unhandled token: {}", operator)
            };

            if let Some(rhs_expression) = parsed_expression.take() {
                let lhs_expression_option =
                    parse_int_ring_expression_from_tokens_rec(tokens, parsed_expression, has_open_parenthesis)?;

                if lhs_expression_option.is_none() {
                    return create_err(format_args!("Missing left hand side expression for operator"), position, Unspecified);
                }

                let mut lhs_expression = lhs_expression_option.unwrap();

                let mut operator_expression = construct_expression(
                    ExpressionComponent::new_int_element(0), // dummy value
                    rhs_expression);

                if lhs_expression.is_operator()
                    && lhs_expression.precedence() < operator_expression.precedence() {
                    swap(operator_expression.left_mut(), lhs_expression.right_mut());
                    swap(lhs_expression.right_mut(), &mut operator_expression);
                    Ok(Some(lhs_expression))
                } else {
                    swap(operator_expression.left_mut(), &mut lhs_expression);
                    Ok(Some(operator_expression))
                }
            } else {
                return create_err(format_args!("Missing right hand side expression for operator"), position, Unspecified)
            }
        },
        IntRingToken::RightParenthesis => {
            tokens.next();
            if let Some(inner) = parse_int_ring_expression_from_tokens_rec(tokens, parsed_expression, true)? {
                if let Some(IntRingToken::LeftParenthesis) = tokens.next().map(|twp| &twp.token) {
                    parsed_expression.replace(ExpressionComponent::new_parenteses(inner));
                    parse_int_ring_expression_from_tokens_rec(tokens, parsed_expression, has_open_parenthesis)
                } else {
                    create_err(format_args!("Missing left parenthesis for right parenthesis"), position, Unspecified)
                }
            } else {
                create_err(format_args!("No expression"), position, NoExpression)
            }
        }
        IntRingToken::LeftParenthesis if has_open_parenthesis => Ok(None),
        IntRingToken::LeftParenthesis if !has_open_parenthesis => create_err(format_args!("Missing right parenthesis for left parenthesis"), position, Unspecified),
        _ => create_err(format_args!("Unhandled token: {}", token), position, Unspecified)
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::ring::intring::{IntRingElement};
    use crate::expression::{ExpressionComponent};
    use crate::expression::parser::{parse_int_ring_expression, ParseExpressionError};
    use crate::expression::parser::ParseExpressionErrorKind::{NoExpression, TokenParseError, Unspecified};

    #[test]
    fn simple_value() {
        let expression = parse_int_ring_expression("34").expect("ok");

        assert_eq!(Ok(IntRingElement::new(34)), expression.evaluate());
    }

    #[test]
    fn two_simple_values() {
        let expression_result = parse_int_ring_expression("1 2");

        assert_eq!(Err(ParseExpressionError{message: "Ring element cannot be followed by another ring element in expression".to_string(), position: 0, kind: Unspecified}), expression_result);
    }

    #[test]
    fn empty() {
        let expression_result = parse_int_ring_expression("  ");

        assert_eq!(Err(ParseExpressionError{message: "No expression".to_string(), position: 0, kind: NoExpression}), expression_result);
    }

    #[test]
    fn token_parse_error() {
        let expression_result = parse_int_ring_expression("5 hest");

        assert_eq!(Err(ParseExpressionError{message: "Invalid token".to_string(), position: 2, kind: TokenParseError}), expression_result);
    }

    #[test]
    fn add() {
        let expression = parse_int_ring_expression("2 + 5").expect("ok");

        assert_eq!(Ok(IntRingElement::new(7)), expression.evaluate());
    }

    #[test]
    fn sub() {
        let expression = parse_int_ring_expression("2 - 5").expect("ok");

        assert_eq!(Ok(IntRingElement::new(-3)), expression.evaluate());
    }

    #[test]
    fn mul() {
        let expression = parse_int_ring_expression("2 * 5").expect("ok");

        assert_eq!(Ok(IntRingElement::new(10)), expression.evaluate());
    }

    #[test]
    fn div() {
        let expression = parse_int_ring_expression("6 / 2").expect("ok");

        assert_eq!(Ok(IntRingElement::new(3)), expression.evaluate());
    }

    #[test]
    fn add_missing_rhs() {
        let expression_result = parse_int_ring_expression("2 + ");

        assert_eq!(Err(ParseExpressionError{message: "Missing right hand side expression for operator".to_string(), position: 2, kind: Unspecified}), expression_result);
    }

    #[test]
    fn add_missing_lhs() {
        let expression_result = parse_int_ring_expression(" + 5");

        assert_eq!(Err(ParseExpressionError{message: "Missing left hand side expression for operator".to_string(), position: 1, kind: Unspecified}), expression_result);
    }

    #[test]
    fn add_twice() {
        let expression = parse_int_ring_expression("2 + 5 + 1").expect("ok");

        assert_eq!(Ok(IntRingElement::new(8)), expression.evaluate());
    }

    #[test]
    fn add_left_associative() {
        let expression = parse_int_ring_expression("2 + 5 + 1").expect("ok");

        assert!(matches!(expression, ExpressionComponent::Addition{..}));
        if let ExpressionComponent::Addition{right, ..} = expression {
            assert_eq!(ExpressionComponent::new_int_element(1), *right);
        } else {
            assert!(false, "should be addition");
        }
    }

    #[test]
    fn precedence_structure() {
        let expression = parse_int_ring_expression("2 + 5 * 1").expect("ok");

        assert_eq!(ExpressionComponent::new_addition(
            ExpressionComponent::new_int_element(2),
            ExpressionComponent::new_multiplication(
                ExpressionComponent::new_int_element(5),
                ExpressionComponent::new_int_element(1))
        ), expression);

        assert_eq!(Ok(IntRingElement::new(7)), expression.evaluate())
    }

    #[test]
    fn precedence_structure2() {
        let expression = parse_int_ring_expression("2 + 5 * 1 * 3").expect("ok");

        assert_eq!(ExpressionComponent::new_addition(
            ExpressionComponent::new_int_element(2),
            ExpressionComponent::new_multiplication(
                ExpressionComponent::new_multiplication(
                    ExpressionComponent::new_int_element(5),
                    ExpressionComponent::new_int_element(1)),
                ExpressionComponent::new_int_element(3))
        ), expression);

        assert_eq!(Ok(IntRingElement::new(2 + 5 * 1 * 3)), expression.evaluate())
    }

    #[test]
    fn precedence_structure_parentheses() {
        let expression = parse_int_ring_expression("(2 + 5) * 1 * 3").expect("ok");

        assert_eq!(ExpressionComponent::new_multiplication(
            ExpressionComponent::new_multiplication(
                ExpressionComponent::new_parenteses(ExpressionComponent::new_addition(
                    ExpressionComponent::new_int_element(2),
                    ExpressionComponent::new_int_element(5))),
                ExpressionComponent::new_int_element(1)),
            ExpressionComponent::new_int_element(3),
        ), expression);

        assert_eq!(Ok(IntRingElement::new((2 + 5) * 1 * 3)), expression.evaluate())
    }

    #[test]
    fn precedence_structure_parentheses2() {
        let expression = parse_int_ring_expression("(2 + (5)) * 1 * (3 + 4)").expect("ok");

        assert_eq!(ExpressionComponent::new_multiplication(
            ExpressionComponent::new_multiplication(
                ExpressionComponent::new_parenteses(ExpressionComponent::new_addition(
                    ExpressionComponent::new_int_element(2),
                    ExpressionComponent::new_parenteses(ExpressionComponent::new_int_element(5)))),
                ExpressionComponent::new_int_element(1)),
            ExpressionComponent::new_parenteses(
                ExpressionComponent::new_addition(
                    ExpressionComponent::new_int_element(3),
                    ExpressionComponent::new_int_element(4),
                ))

        ), expression);

        assert_eq!(Ok(IntRingElement::new((2 + (5)) * 1 * (3 + 4))), expression.evaluate())
    }

    #[test]
    fn add_lower_precedence_than_mul() {
        let expression = parse_int_ring_expression("2 * 5 + 1").expect("ok");

        assert!(matches!(expression, ExpressionComponent::Addition{..}));
        if let ExpressionComponent::Addition{right, ..} = expression {
            assert_eq!(ExpressionComponent::new_int_element(1), *right);
        } else {
            assert!(false, "should be addition");
        }
    }

    #[test]
    fn mul_higher_precedence_than_add() {
        let expression = parse_int_ring_expression("2 + 5 * 1").expect("ok");

        assert!(matches!(expression, ExpressionComponent::Addition{..}));
        if let ExpressionComponent::Addition{left, ..} = expression {
            assert_eq!(ExpressionComponent::new_int_element(2), *left);
        } else {
            assert!(false, "should be addition");
        }
    }

    #[test]
    fn div_higher_precedence_than_add() {
        let expression = parse_int_ring_expression("2 + 5 / 1").expect("ok");

        assert!(matches!(expression, ExpressionComponent::Addition{..}));
        if let ExpressionComponent::Addition{left, ..} = expression {
            assert_eq!(ExpressionComponent::new_int_element(2), *left);
        } else {
            assert!(false, "should be addition");
        }
    }

    #[test]
    fn mul_higher_precedence_than_sub() {
        let expression = parse_int_ring_expression("2 - 5 * 1").expect("ok");

        assert!(matches!(expression, ExpressionComponent::Subtraction{..}));
        if let ExpressionComponent::Subtraction{left, ..} = expression {
            assert_eq!(ExpressionComponent::new_int_element(2), *left);
        } else {
            assert!(false, "should be subtraction");
        }
    }

    #[test]
    fn div_higher_precedence_than_sub() {
        let expression = parse_int_ring_expression("2 - 5 / 1").expect("ok");

        assert!(matches!(expression, ExpressionComponent::Subtraction{..}));
        if let ExpressionComponent::Subtraction{left, ..} = expression {
            assert_eq!(ExpressionComponent::new_int_element(2), *left);
        } else {
            assert!(false, "should be subtraction");
        }
    }

    #[test]
    fn missing_left_parenthesis() {
        let expression_result = parse_int_ring_expression("3 + 5)");

        assert_eq!(Err(ParseExpressionError{message: "Missing left parenthesis for right parenthesis".to_string(), position: 5, kind: Unspecified}), expression_result);
    }

    #[test]
    fn missing_left_parenthesis2() {
        let expression_result = parse_int_ring_expression("(3 + 5))");

        assert_eq!(Err(ParseExpressionError{message: "Missing left parenthesis for right parenthesis".to_string(), position: 7, kind: Unspecified}), expression_result);
    }

    #[test]
    fn missing_right_parenthesis() {
        let expression_result = parse_int_ring_expression("3 + (3 + 5");

        assert_eq!(Err(ParseExpressionError{message: "Missing right parenthesis for left parenthesis".to_string(), position: 4, kind: Unspecified}), expression_result);
    }

    #[test]
    fn missing_right_parenthesis2() {
        let expression_result = parse_int_ring_expression("(3 + (3 + 5)");

        assert_eq!(Err(ParseExpressionError{message: "Missing right parenthesis for left parenthesis".to_string(), position: 0, kind: Unspecified}), expression_result);
    }

    #[test]
    fn emtpy_expression_in_parenthesis() {
        let expression_result = parse_int_ring_expression("8 + () * 8");

        assert_eq!(Err(ParseExpressionError{message: "No expression".to_string(), position: 5, kind: NoExpression}), expression_result);
    }

    #[test]
    fn unary_minus() {
        let expression = parse_int_ring_expression("2 * (-5)").expect("ok");

        assert_eq!(ExpressionComponent::new_multiplication(
            ExpressionComponent::new_int_element(2),
            ExpressionComponent::new_parenteses(
                ExpressionComponent::new_int_element(-5))
        ), expression);

        assert_eq!(Ok(IntRingElement::new(-10)), expression.evaluate())
    }
}