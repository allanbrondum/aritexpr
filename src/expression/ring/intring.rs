use crate::expression::ring::{Ring, RingResult, RingElement, RingError};
use std::fmt::{Display, Formatter};
use crate::expression::ExpressionComponent;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct IntRingElement {
    value: i64
}

impl Display for IntRingElement {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)?;
        Ok(())
    }
}

impl RingElement for IntRingElement {

}

impl IntRingElement {
    pub fn new(value: i64) -> IntRingElement {
        IntRingElement {
            value
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct IntRing {
}

impl Ring for IntRing {
    type RingElementType = IntRingElement;

    fn add(elm1: &Self::RingElementType, elm2: &Self::RingElementType) -> RingResult<Self::RingElementType> {
        IntRing::ring_result(elm1.value.checked_add(elm2.value))
    }

    fn sub(elm1: &Self::RingElementType, elm2: &Self::RingElementType) -> RingResult<Self::RingElementType> {
        IntRing::ring_result(elm1.value.checked_sub(elm2.value))
    }

    fn mul(elm1: &Self::RingElementType, elm2: &Self::RingElementType) -> RingResult<Self::RingElementType> {
        IntRing::ring_result(elm1.value.checked_mul(elm2.value))
    }

    fn div(elm1: &Self::RingElementType, elm2: &Self::RingElementType) -> RingResult<Self::RingElementType> {
        let rem = elm1.value.checked_rem(elm2.value);
        if let Some(d ) = rem {
            if d != 0 {
                return Err(RingError { message: "Result not in ring".to_string() });
            }
        }
        IntRing::ring_result(elm1.value.checked_div(elm2.value))
    }
}

impl IntRing {
    fn ring_result(res: Option<i64>) -> Result<IntRingElement, RingError> {
        match res {
            Some(val) => Ok(IntRingElement::new(val)),
            None => Err(RingError { message: "Overflow".to_string() }),
        }
    }
}

impl ExpressionComponent<IntRing> {
    pub fn new_int_element(value: i64) -> ExpressionComponent<IntRing> {
        ExpressionComponent::new_ring_element(IntRingElement::new(value))
    }
}


#[cfg(test)]
mod tests {
    use crate::expression::ring::intring::{IntRingElement, IntRing};
    use crate::expression::ring::{Ring, RingError};

    #[test]
    fn add() {
        let elm1 = IntRingElement::new(5);
        let elm2 = IntRingElement::new(-3);

        let res = IntRing::add(&elm1, &elm2);

        assert_eq!(Ok(IntRingElement::new(2)), res);
    }

    #[test]
    fn add_overflow() {
        let elm1 = IntRingElement::new(i64::MAX);
        let elm2 = IntRingElement::new(1);

        let res = IntRing::add(&elm1, &elm2);

        assert_eq!(Err(RingError{message: "Overflow".to_string()}), res);
    }

    #[test]
    fn sub() {
        let elm1 = IntRingElement::new(5);
        let elm2 = IntRingElement::new(2);

        let res = IntRing::sub(&elm1, &elm2);

        assert_eq!(Ok(IntRingElement::new(3)), res);
    }

    #[test]
    fn sub_overflow() {
        let elm1 = IntRingElement::new(i64::MIN);
        let elm2 = IntRingElement::new(1);

        let res = IntRing::sub(&elm1, &elm2);

        assert_eq!(Err(RingError{message: "Overflow".to_string()}), res);
    }

    #[test]
    fn mul() {
        let elm1 = IntRingElement::new(5);
        let elm2 = IntRingElement::new(2);

        let res = IntRing::mul(&elm1, &elm2);

        assert_eq!(Ok(IntRingElement::new(10)), res);
    }

    #[test]
    fn mul2() {
        let elm1 = IntRingElement::new(5);
        let elm2 = IntRingElement::new(-2);

        let res = IntRing::mul(&elm1, &elm2);

        assert_eq!(Ok(IntRingElement::new(-10)), res);
    }

    #[test]
    fn mul_overflow() {
        let elm1 = IntRingElement::new(i64::MAX);
        let elm2 = IntRingElement::new(2);

        let res = IntRing::mul(&elm1, &elm2);

        assert_eq!(Err(RingError{message: "Overflow".to_string()}), res);
    }

    #[test]
    fn div1() {
        let elm1 = IntRingElement::new(6);
        let elm2 = IntRingElement::new(2);

        let res = IntRing::div(&elm1, &elm2);

        assert_eq!(Ok(IntRingElement::new(3)), res);
    }

    #[test]
    fn div2() {
        let elm1 = IntRingElement::new(-6);
        let elm2 = IntRingElement::new(2);

        let res = IntRing::div(&elm1, &elm2);

        assert_eq!(Ok(IntRingElement::new(-3)), res);
    }

    #[test]
    fn div3() {
        let elm1 = IntRingElement::new(6);
        let elm2 = IntRingElement::new(-2);

        let res = IntRing::div(&elm1, &elm2);

        assert_eq!(Ok(IntRingElement::new(-3)), res);
    }

    #[test]
    fn div_zero() {
        let elm1 = IntRingElement::new(2);
        let elm2 = IntRingElement::new(0);

        let res = IntRing::div(&elm1, &elm2);

        assert_eq!(Err(RingError{message: "Overflow".to_string()}), res);
    }

    #[test]
    fn div_zero2() {
        let elm1 = IntRingElement::new(0);
        let elm2 = IntRingElement::new(0);

        let res = IntRing::div(&elm1, &elm2);

        assert_eq!(Err(RingError{message: "Overflow".to_string()}), res);
    }

    #[test]
    fn div_not_int() {
        let elm1 = IntRingElement::new(5);
        let elm2 = IntRingElement::new(2);

        let res = IntRing::div(&elm1, &elm2);

        assert_eq!(Err(RingError{message: "Result not in ring".to_string()}), res);
    }

    #[test]
    fn div_not_int2() {
        let elm1 = IntRingElement::new(-5);
        let elm2 = IntRingElement::new(2);

        let res = IntRing::div(&elm1, &elm2);

        assert_eq!(Err(RingError{message: "Result not in ring".to_string()}), res);
    }

    #[test]
    fn div_not_int3() {
        let elm1 = IntRingElement::new(5);
        let elm2 = IntRingElement::new(-2);

        let res = IntRing::div(&elm1, &elm2);

        assert_eq!(Err(RingError{message: "Result not in ring".to_string()}), res);
    }
}