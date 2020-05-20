use crate::ux::{Element, Elements, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Failed(Element),
}

/// A generic expressible property
pub trait Property {
    /// Get the result of this property
    fn result(&self) -> Outcome;

    /// Simple logical And combinator, this property and the next one must pass to pass
    fn and<O>(self, other: O) -> And<Self, O>
    where
        Self: Sized,
    {
        And {
            prop_a: self,
            prop_b: other,
        }
    }

    /*
    fn and_then<F>(self, other: F) -> AndThen<Self, F>
    where
        Self: Sized,
        F: Fn() -> BoxProperty,
    {
        AndThen {
            prop_a: self,
            prop_b: other,
        }
    }
    */

    /// Simple logical Or combinator, this property or the next one must pass to pass
    fn or<O>(self, other: O) -> Or<Self, O>
    where
        Self: Sized,
    {
        Or {
            prop_a: self,
            prop_b: other,
        }
    }
}

/// A Generic Boxed Property
pub struct BoxProperty(Box<dyn Property>);

/// Logical And between properties
pub struct And<A, B> {
    prop_a: A,
    prop_b: B,
}

impl<A, B> Property for And<A, B>
where
    A: Property,
    B: Property,
{
    fn result(&self) -> Outcome {
        fn failure_element(left: Value, right: Value) -> Outcome {
            let mut output = Elements::new();
            output.append("left", left);
            output.append("right", right);
            Outcome::Failed(Element::new("and", output.into()))
        }
        match (self.prop_a.result(), self.prop_b.result()) {
            (Outcome::Passed, Outcome::Passed) => Outcome::Passed,
            (Outcome::Failed(f1), Outcome::Passed) => {
                failure_element(Value::sub(f1), "passed".into())
            }
            (Outcome::Passed, Outcome::Failed(f2)) => {
                failure_element("passed".into(), Value::sub(f2))
            }
            (Outcome::Failed(f1), Outcome::Failed(f2)) => {
                failure_element(Value::sub(f1), Value::sub(f2))
            }
        }
    }
}

/*
pub struct AndThen<A, B> {
    prop_a: A,
    prop_b: B,
}

impl<A, B> Property for AndThen<A, B> {}
*/

/// Logical Or between properties
pub struct Or<A, B> {
    prop_a: A,
    prop_b: B,
}

impl<A, B> Property for Or<A, B>
where
    A: Property,
    B: Property,
{
    fn result(&self) -> Outcome {
        match (self.prop_a.result(), self.prop_b.result()) {
            (Outcome::Passed, _) => Outcome::Passed,
            (_, Outcome::Passed) => Outcome::Passed,
            (Outcome::Failed(f1), Outcome::Failed(f2)) => {
                let mut output = Elements::new();
                output.append("left", Value::sub(f1));
                output.append("right", Value::sub(f2));
                Outcome::Failed(Element::new("or", output.into()))
            }
        }
    }
}
