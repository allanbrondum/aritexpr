use std::fmt::{Formatter, Display};
use core::fmt;
use std::{result, error};
use std::hash::Hash;

pub mod intring;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct RingError {
    pub message: String
}

impl fmt::Display for RingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)?;
        Ok(())
    }
}

impl error::Error for RingError {
}

pub type RingResult<T> = result::Result<T, RingError>;

pub trait RingElement : Display + PartialEq + Eq + Hash + Clone {
}

/// Represents ring or class of rings with division. Arithmetic operations in the ring are allowed to fail.
pub trait Ring {
    type RingElementType : RingElement;

    fn add(elm1: &Self::RingElementType, elm2: &Self::RingElementType) -> RingResult<Self::RingElementType>;
    fn sub(elm1: &Self::RingElementType, elm2: &Self::RingElementType) -> RingResult<Self::RingElementType>;
    fn mul(elm1: &Self::RingElementType, elm2: &Self::RingElementType) -> RingResult<Self::RingElementType>;
    fn div(elm1: &Self::RingElementType, elm2: &Self::RingElementType) -> RingResult<Self::RingElementType>;

}