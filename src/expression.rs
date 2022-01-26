use std::fmt::{Formatter};
use std::{error, result};
use core::fmt;
use crate::expression::ring::{Ring, RingError, RingResult};
use crate::expression::ExpressionComponent::{RingElement, Addition, Subtraction, Multiplication, Division, Parentheses, UnaryMinus};
use std::ops::DerefMut;

pub mod ring;
pub mod parser;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct EvaluateExpressionError {
    pub message: String,
    // pub position: usize
}

impl fmt::Display for EvaluateExpressionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error evaluating expression: {}", self.message)
    }
}

impl error::Error for EvaluateExpressionError {
}

impl From<RingError> for EvaluateExpressionError {
    fn from(err: RingError) -> Self {
        EvaluateExpressionError {
            message: err.message
        }
    }
}

pub type EvaluateExpressionResult<T> = result::Result<T, EvaluateExpressionError>;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ExpressionComponent<R: Ring> {
    RingElement(R::RingElementType),
    Parentheses(Box<ExpressionComponent<R>>),
    UnaryMinus(Box<ExpressionComponent<R>>),
    Addition {
        left: Box<ExpressionComponent<R>>,
        right: Box<ExpressionComponent<R>>
    },
    Subtraction {
        left: Box<ExpressionComponent<R>>,
        right: Box<ExpressionComponent<R>>
    },
    Multiplication {
        left: Box<ExpressionComponent<R>>,
        right: Box<ExpressionComponent<R>>
    },
    Division {
        left: Box<ExpressionComponent<R>>,
        right: Box<ExpressionComponent<R>>
    },
}

impl<R: Ring> ExpressionComponent<R> {
    pub fn new_ring_element(element: R::RingElementType) -> ExpressionComponent<R> {
        RingElement(element)
    }

    pub fn new_addition(expr1: Self, expr2: Self) -> ExpressionComponent<R> {
        Addition {
            left: Box::new(expr1),
            right: Box::new(expr2)
        }
    }

    pub fn new_subtraction(expr1: Self, expr2: Self) -> ExpressionComponent<R> {
        Subtraction {
            left: Box::new(expr1),
            right: Box::new(expr2)
        }
    }

    pub fn new_multiplication(expr1: Self, expr2: Self) -> ExpressionComponent<R> {
        Multiplication {
            left: Box::new(expr1),
            right: Box::new(expr2)
        }
    }

    pub fn new_division(expr1: Self, expr2: Self) -> ExpressionComponent<R> {
        Division {
            left: Box::new(expr1),
            right: Box::new(expr2)
        }
    }

    pub fn new_parenteses(expr: Self) -> ExpressionComponent<R> {
        Parentheses(Box::new(expr))
    }

    pub fn new_unary_minus(expr: Self) -> ExpressionComponent<R> {
        UnaryMinus(Box::new(expr))
    }

    fn is_operator(&self) -> bool {
        match self {
            RingElement(_) => false,
            Addition { .. } => true,
            Subtraction { .. } => true,
            Multiplication { .. } => true,
            Division { .. } => true,
            Parentheses(_) => false,
            UnaryMinus(_) => false,
        }
    }

    fn precedence(&self) -> i32 {
        match self {
            RingElement(_) => i32::MAX,
            Parentheses(_) => i32::MAX,
            UnaryMinus(_) => i32::MAX,
            Addition { .. } => 0,
            Subtraction { .. } => 0,
            Multiplication { .. } => 1,
            Division { .. } => 1,
        }
    }

    fn left_mut(&mut self) -> &mut ExpressionComponent<R> {
        match self {
            ExpressionComponent::Addition { left, .. } => left.deref_mut(),
            ExpressionComponent::Subtraction { left, .. } => left.deref_mut(),
            ExpressionComponent::Multiplication { left, .. } => left.deref_mut(),
            ExpressionComponent::Division { left, .. } => left.deref_mut(),
            _ => panic!("Not an operator"),
        }
    }

    fn right_mut(&mut self) -> &mut ExpressionComponent<R> {
        match self {
            ExpressionComponent::Addition { right, .. } => right.deref_mut(),
            ExpressionComponent::Subtraction { right, .. } => right.deref_mut(),
            ExpressionComponent::Multiplication { right, .. } => right.deref_mut(),
            ExpressionComponent::Division { right, .. } => right.deref_mut(),
            _ => panic!("Not an operator"),
        }
    }
}

impl<R: Ring> ExpressionComponent<R> {
    pub fn evaluate(&self) -> EvaluateExpressionResult<R::RingElementType> {
        match self {
            RingElement(r) => Ok(r.clone()),
            Parentheses(inner) => inner.evaluate(),
            UnaryMinus(inner) => panic!("implement"),
            Addition {left, right} => {
                Self::evaluate_binary_operation(R::add, &left, &right)
            }
            Subtraction {left, right} => {
                Self::evaluate_binary_operation(R::sub, &left, &right)
            }
            Multiplication {left, right} => {
                Self::evaluate_binary_operation(R::mul, &left, &right)
            }
            Division {left, right} => {
                Self::evaluate_binary_operation(R::div, &left, &right)
            }
        }
    }

    fn evaluate_binary_operation(
        binary_operation: fn(&R::RingElementType, &R::RingElementType) -> RingResult<R::RingElementType>,
        left: &Box<ExpressionComponent<R>>,
        right: &Box<ExpressionComponent<R>>) -> EvaluateExpressionResult<R::RingElementType>
    {
        Ok(binary_operation(&left.evaluate()?, &right.evaluate()?)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::ring::intring::{IntRingElement, IntRing};
    use crate::expression::{ExpressionComponent, EvaluateExpressionError};

    #[test]
    fn simple_value() {
        let element = IntRingElement::new(5);
        let expression = ExpressionComponent::<IntRing>::new_ring_element(element.clone());

        assert_eq!(Ok(element), expression.evaluate());
    }

    #[test]
    fn addition() {
        let expression =
            ExpressionComponent::<IntRing>::new_addition(
                ExpressionComponent::new_ring_element(IntRingElement::new(5)),
                ExpressionComponent::new_ring_element(IntRingElement::new(7)));

        assert_eq!(Ok(IntRingElement::new(12)), expression.evaluate());
    }

    #[test]
    fn addition_overflow() {
        let expression =
            ExpressionComponent::<IntRing>::new_addition(
                ExpressionComponent::new_ring_element(IntRingElement::new(i64::MAX)),
                ExpressionComponent::new_ring_element(IntRingElement::new(7)));

        assert_eq!(Err(EvaluateExpressionError {message: "Overflow".to_string()}), expression.evaluate());
    }

    #[test]
    fn subtraction() {
        let expression =
            ExpressionComponent::<IntRing>::new_subtraction(
                ExpressionComponent::new_ring_element(IntRingElement::new(5)),
                ExpressionComponent::new_ring_element(IntRingElement::new(7)));

        assert_eq!(Ok(IntRingElement::new(-2)), expression.evaluate());
    }

    #[test]
    fn multiplication() {
        let expression =
            ExpressionComponent::<IntRing>::new_multiplication(
                ExpressionComponent::new_ring_element(IntRingElement::new(5)),
                ExpressionComponent::new_ring_element(IntRingElement::new(7)));

        assert_eq!(Ok(IntRingElement::new(35)), expression.evaluate());
    }

    #[test]
    fn division() {
        let expression =
            ExpressionComponent::<IntRing>::new_division(
                ExpressionComponent::new_ring_element(IntRingElement::new(6)),
                ExpressionComponent::new_ring_element(IntRingElement::new(2)));

        assert_eq!(Ok(IntRingElement::new(3)), expression.evaluate());
    }

    #[test]
    fn parenthesis() {
        let expression =
            ExpressionComponent::<IntRing>::new_parenteses(
                ExpressionComponent::new_ring_element(IntRingElement::new(5)));

        assert_eq!(Ok(IntRingElement::new(5)), expression.evaluate());
    }

}