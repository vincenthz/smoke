//! Smoke framework is a software testing library. The name is
//! linked to the "smoke testing", although this library does
//! more than just "smoke testing".
//!
//! Smoke framework is composed of 3 sub frameworks:
//!
//! * Tests framework : Various testing strategy
//! * Generators framework : generate arbitrary values following generation rules
//! * Runtime : Execution of generation and tests
//!
//! The tests and generator frameworks can be used independently

pub mod generator;
mod rand;
mod ux;

pub use generator::Generator;
pub use rand::{NumPrimitive, Seed, R};
