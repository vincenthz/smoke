use super::generator::Generator;
use super::initonce::InitOnce;
use super::property::{self, Property};
use super::rand::Seed;
use super::ux::{TestResults, TestRunStatus};
use super::R;
use std::panic::{catch_unwind, set_hook, take_hook, AssertUnwindSafe, PanicInfo};
use std::time::{Duration, SystemTime};

const ENV_SEED: &str = "SMOKE_SEED";

pub struct PanicError(String);

use crate::generator::SuchThatRetryFailure;

use std::fmt;

static INSTANCE_SEED: InitOnce<Seed> = InitOnce::init();

fn run_catch_panic<F, R>(f: F) -> Result<R, PanicError>
where
    F: FnOnce() -> R,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Err(e) => {
            if let Some(SuchThatRetryFailure) = e.downcast_ref::<SuchThatRetryFailure>() {
                Err(PanicError("such that retry failure".to_string()))
            } else if let Some(e) = e.downcast_ref::<&'static str>() {
                Err(PanicError((*e).to_string()))
            } else if let Some(e) = e.downcast_ref::<String>() {
                Err(PanicError(e.clone()))
            } else {
                Err(PanicError("unknown type of panic error".to_string()))
            }
        }
        Ok(prop_result) => Ok(prop_result),
    }
}

#[derive(Clone)]
pub struct Forall<G> {
    generator: G,
}

impl<G> Forall<G> {
    pub fn ensure<T, P, F>(self, f: F) -> Ensure<G, F>
    where
        G: Generator<Item = T>,
        P: Property,
        F: Fn(&T) -> P,
        T: fmt::Debug + Clone + 'static,
    {
        Ensure {
            generator: self.generator,
            property_closure: f,
        }
    }
}

pub fn forall<T, G>(g: G) -> Forall<G>
where
    G: Generator<Item = T>,
{
    Forall { generator: g }
}

/// Execution context
pub struct Context {
    seed: Seed,
    test_results: TestResults,
}

pub struct Ensure<G: Generator, F> {
    generator: G,
    property_closure: F,
}

pub trait Testable {
    fn test(self, context: &mut Context);
}

impl<T, G, F, P> Testable for Ensure<G, F>
where
    G: Generator<Item = T>,
    P: Property,
    F: Fn(&T) -> P,
    T: fmt::Debug + 'static,
{
    fn test(self, context: &mut Context) {
        let mut r = R::from_seed(context.seed);

        println!("{:?}", std::env::args().collect::<Vec<_>>());

        let nb_tests = 1000;

        let start = SystemTime::now();

        let mut result = TestResults::new();

        let generator = self.generator;
        let property_closure = self.property_closure;
        for _ in 0..nb_tests {
            let mut test_rng = r.sub();

            let input = generator.gen(&mut test_rng);
            let to_report = &input;
            //println!("item: {:?}", v);
            match run_catch_panic(|| property_closure(&input)) {
                Err(PanicError(p)) => {
                    result.add_failed(format!("input: {:?}\npanic: \"{}\"\n", to_report, p))
                }
                Ok(p) => match p.result() {
                    property::Outcome::Passed => result.add_success(),
                    property::Outcome::Failed(t) => result.add_failed(format!(
                        "input = {:?}\nproperty failed:\n{}",
                        to_report,
                        t.display(2),
                    )),
                },
            }
        }
        let finished = SystemTime::now();
        let duration = finished
            .duration_since(start)
            .unwrap_or_else(|_| Duration::default());
        result.set_duration(duration);
        context.test_results.add_subtests(&result);
    }
}

impl Context {
    #[allow(clippy::new_without_default)]
    fn new() -> Self {
        use std::str::FromStr;
        let seed = match std::env::var(ENV_SEED) {
            Ok(v) => Seed::from_str(&v).expect("invalid seed format"),
            Err(_) => *INSTANCE_SEED.load(Seed::new),
        };
        Self {
            seed,
            test_results: TestResults::new(),
        }
    }
}

pub fn run<F>(f: F)
where
    F: Fn(&mut Context) -> (),
{
    let mut ctx = Context::new();

    fn dont_print_panic(_: &PanicInfo) {};

    set_hook(Box::new(dont_print_panic));

    // execute the user tests
    f(&mut ctx);

    let _ = take_hook();

    // print result
    let tr = ctx.test_results;
    match tr.to_status() {
        TestRunStatus::Passed => println!("Passed {} tests", tr.nb_tests),
        TestRunStatus::Skipped => {}
        TestRunStatus::Failed => {
            for (i, failure) in tr.failures.iter().enumerate() {
                println!("# Failure {}\n{}", i, failure)
            }
            panic!(
                "\n{:?} tests failed / {:?} tests runned",
                tr.nb_failed, tr.nb_tests
            );
        }
    }
}
