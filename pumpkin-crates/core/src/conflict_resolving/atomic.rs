use std::rc::Rc;

use drcp_format::IntAtomic;
use pumpkin_checking::AtomicConstraint;
use pumpkin_checking::Comparison;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Atomic {
    True,
    False,
    IntAtomic(IntAtomic<Rc<str>, i32>),
}

impl From<IntAtomic<Rc<str>, i32>> for Atomic {
    fn from(value: IntAtomic<Rc<str>, i32>) -> Self {
        Atomic::IntAtomic(value)
    }
}

impl From<bool> for Atomic {
    fn from(value: bool) -> Self {
        if value { Atomic::True } else { Atomic::False }
    }
}

impl Atomic {
    pub fn set_value(&mut self, value: i32) {
        match self {
            Atomic::True | Atomic::False => {}
            Atomic::IntAtomic(int_atomic) => int_atomic.value = value,
        }
    }
}

impl AtomicConstraint for Atomic {
    type Identifier = Rc<str>;

    fn identifier(&self) -> Self::Identifier {
        match self {
            Atomic::True => Rc::from("true"),
            Atomic::False => Rc::from("false"),
            Atomic::IntAtomic(int_atomic) => Rc::clone(&int_atomic.name),
        }
    }

    fn comparison(&self) -> Comparison {
        let Atomic::IntAtomic(int_atomic) = self else {
            return Comparison::Equal;
        };

        match int_atomic.comparison {
            drcp_format::IntComparison::GreaterEqual => Comparison::GreaterEqual,
            drcp_format::IntComparison::LessEqual => Comparison::LessEqual,
            drcp_format::IntComparison::Equal => Comparison::Equal,
            drcp_format::IntComparison::NotEqual => Comparison::NotEqual,
        }
    }

    fn value(&self) -> i32 {
        match self {
            Atomic::True => 1,
            Atomic::False => 0,
            Atomic::IntAtomic(int_atomic) => int_atomic.value,
        }
    }

    fn negate(&self) -> Self {
        match self {
            Atomic::True => Atomic::False,
            Atomic::False => Atomic::True,
            Atomic::IntAtomic(int_atomic) => {
                let owned = int_atomic.clone();
                Atomic::IntAtomic(!owned)
            }
        }
    }
}
