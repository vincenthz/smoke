use super::generator::Generator;
use super::initonce::InitOnce;
use super::property::{self, Property};
use super::rand::Seed;
use super::ux::{TestResults, TestRunStatus};
use super::R;
use std::panic::{catch_unwind, set_hook, take_hook, AssertUnwindSafe, PanicInfo};
use std::time::{Duration, SystemTime};

const DEFAULT_NB_TESTS: u64 = 1_000;

const ENV_SEED: &str = "SMOKE_SEED";
const ENV_NB_TESTS: &str = "SMOKE_NB_TESTS";
const ENV_NO_PANIC_CATCH: &str = "SMOKE_NO_PANIC_CATCH";

pub struct PanicError(String);

use crate::generator::SuchThatRetryFailure;

use std::fmt;

static INSTANCE_SEED: InitOnce<Seed> = InitOnce::init();

fn run_catch_panic<F, R>(f: F) -> Result<R, PanicError>
where
    F: FnOnce() -> R,
{
    let no_catch_panic = std::env::var(ENV_NO_PANIC_CATCH).is_ok();
    if no_catch_panic {
        Ok(f())
    } else {
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

/// Put a generator in random sampling mode for property testing
///
/// ```
/// use smoke::{Generator, generator::num, property::equal, forall};
///
/// let property_equal = forall(num::<u32>()).ensure(|x| equal(*x, *x));
/// ```
///
pub fn forall<T, G>(g: G) -> Forall<G>
where
    G: Generator<Item = T>,
{
    Forall { generator: g }
}

/// Execution context
pub struct Context {
    seed: Seed,
    nb_tests: u64,
    test_results: TestResults,
}

/// A testable statement binding a generator with a property
pub struct Ensure<G: Generator, F> {
    generator: G,
    property_closure: F,
}

/// Any tests to run with a testing context
pub trait Testable {
    fn test(&self, context: &Context) -> TestResults;

    fn run(&self, context: &mut Context) {
        let results = self.test(context);
        context.test_results.add_subtests(&results);
    }
}

impl<T, G, F, P> Testable for Ensure<G, F>
where
    G: Generator<Item = T>,
    P: Property,
    F: Fn(&T) -> P,
    T: fmt::Debug + 'static,
{
    fn test(&self, context: &Context) -> TestResults {
        let mut r = R::from_seed(context.seed);

        let nb_tests = context.nb_tests;

        let start = SystemTime::now();

        let mut result = TestResults::new();

        let generator = &self.generator;
        let property_closure = &self.property_closure;
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
        result
    }
}

impl Context {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        use std::str::FromStr;
        let seed = match std::env::var(ENV_SEED) {
            Ok(v) => Seed::from_str(&v).expect("invalid seed format"),
            Err(_) => *INSTANCE_SEED.load(Seed::generate),
        };
        let nb_tests = match std::env::var(ENV_NB_TESTS) {
            Ok(v) => v.parse().expect("invalid seed format"),
            Err(_) => DEFAULT_NB_TESTS,
        };
        Self {
            seed,
            nb_tests,
            test_results: TestResults::new(),
        }
    }

    pub fn seed(&self) -> Seed {
        self.seed
    }

    pub fn set_seed(&mut self, seed: Seed) {
        self.seed = seed;
    }

    pub fn nb_tests(&self) -> u64 {
        self.nb_tests
    }

    pub fn set_nb_tests(&mut self, nb_tests: u64) {
        self.nb_tests = nb_tests;
    }
}

/// Create a new context to execute tests into
///
/// ```
/// use smoke::{run, forall, Generator, Property, Testable, generator::num, property::greater};
///
/// run(|ctx| {
///     forall(num::<u32>())
///         .ensure(|n| greater(*n + 1, *n))
///         .run(ctx);
///     // other test instances
/// });
/// ```
///
pub fn run<F>(f: F)
where
    F: Fn(&mut Context) -> (),
{
    let mut ctx = Context::new();

    fn dont_print_panic(_: &PanicInfo) {}

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
