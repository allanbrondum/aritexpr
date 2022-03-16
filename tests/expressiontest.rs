use aritexpr::expression::parser::parse_int_ring_expression;
use aritexpr::expression::ring::intring::IntRingElement;

#[test]
fn expression() {
    let expression = parse_int_ring_expression("2 + 5").expect("ok");

    assert_eq!(Ok(IntRingElement::new(7)), expression.evaluate());
}
