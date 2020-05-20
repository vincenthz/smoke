use super::api::{Outcome, Property};
use crate::ux::{Element, Elements};
use std::cmp::Ordering;

struct NamedOp<T> {
    name: &'static str,
    op: fn(T) -> bool,
}

const EQ_OP: NamedOp<bool> = NamedOp {
    name: "==",
    op: |b| b,
};
const NE_OP: NamedOp<bool> = NamedOp {
    name: "!=",
    op: |b| !b,
};
const GT_OP: NamedOp<Ordering> = NamedOp {
    name: ">",
    op: |o| o == Ordering::Greater,
};
const GE_OP: NamedOp<Ordering> = NamedOp {
    name: ">=",
    op: |o| o == Ordering::Greater || o == Ordering::Equal,
};
const LT_OP: NamedOp<Ordering> = NamedOp {
    name: "<",
    op: |o| o == Ordering::Less,
};
const LE_OP: NamedOp<Ordering> = NamedOp {
    name: "<=",
    op: |o| o == Ordering::Less || o == Ordering::Equal,
};

/// Relation between 2 values based on the Eq trait
pub struct RelationEq<T> {
    left: T,
    right: T,
    op: &'static NamedOp<bool>,
}

impl<T: Eq + std::fmt::Debug> Property for RelationEq<T> {
    fn result(&self) -> Outcome {
        if (self.op.op)(self.left == self.right) {
            Outcome::Passed
        } else {
            let mut output = Elements::new();
            let l_value = format!("{:?}", self.left);
            let r_value = format!("{:?}", self.right);
            output.append("left", l_value.into());
            output.append("right", r_value.into());
            Outcome::Failed(Element::new(self.op.name, output.into()))
        }
    }
}

/// Relation between 2 values based on the Ord trait
pub struct RelationOrd<T> {
    left: T,
    right: T,
    op: &'static NamedOp<Ordering>,
}

impl<T: Ord + std::fmt::Debug> Property for RelationOrd<T> {
    fn result(&self) -> Outcome {
        if (self.op.op)(self.left.cmp(&self.right)) {
            Outcome::Passed
        } else {
            let mut output = Elements::new();
            let l_value = format!("{:?}", self.left);
            let r_value = format!("{:?}", self.right);
            output.append("left", l_value.into());
            output.append("right", r_value.into());
            Outcome::Failed(Element::new(self.op.name, output.into()))
        }
    }
}

/// Check that 2 elements are equal
pub fn equal<T: Eq>(left: T, right: T) -> RelationEq<T> {
    RelationEq {
        left,
        right,
        op: &EQ_OP,
    }
}

/// Check that 2 elements are not equal
pub fn not_equal<T: Eq>(left: T, right: T) -> RelationEq<T> {
    RelationEq {
        left,
        right,
        op: &NE_OP,
    }
}

/// Check that the left element is greater than the right element
pub fn greater<T: Ord>(left: T, right: T) -> RelationOrd<T> {
    RelationOrd {
        left,
        right,
        op: &GT_OP,
    }
}

/// Check that the left element is greater or equal than the right element
pub fn greater_equal<T: Ord>(left: T, right: T) -> RelationOrd<T> {
    RelationOrd {
        left,
        right,
        op: &GE_OP,
    }
}

/// Check that the left element is less than the right element
pub fn less<T: Ord>(left: T, right: T) -> RelationOrd<T> {
    RelationOrd {
        left,
        right,
        op: &LT_OP,
    }
}

/// Check that the left element is less or equal than the right element
pub fn less_equal<T: Ord>(left: T, right: T) -> RelationOrd<T> {
    RelationOrd {
        left,
        right,
        op: &LE_OP,
    }
}
