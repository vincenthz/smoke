use std::time::Duration;

/// A key-value pair reporting element
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element(String, Value);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elements(Vec<Element>);

fn display_element(output: &mut String, indent: usize, element: &Element) {
    let Element(k, v) = element;
    //for Element(k, v) in elements.iter() {
    for _ in 0..indent {
        output.push(' ');
    }
    output.push_str(&k);
    output.push_str(": ");
    match v {
        Value::Tree(tree) => {
            // replace recursion instead of smashing the stack
            output.push_str("\n");
            for el in tree.0.iter() {
                display_element(output, indent + 2, el)
            }
        }
        Value::Str(s) => {
            output.push_str(s);
            output.push_str("\n");
        }
    }
}

impl Default for Elements {
    fn default() -> Self {
        Elements(Vec::new())
    }
}

impl Elements {
    pub fn new() -> Self {
        Elements(Vec::new())
    }

    pub fn append(&mut self, key: &str, value: Value) {
        self.0.push(Element::new(key, value))
    }

    pub fn display(&self, indent: usize) -> String {
        let mut output = String::new();
        for element in self.0.iter() {
            display_element(&mut output, indent, element)
        }
        output
    }
}

impl Element {
    pub fn new(key: &str, v: Value) -> Self {
        Element(key.to_string(), v)
    }

    pub fn display(&self, indent: usize) -> String {
        let mut output = String::new();
        display_element(&mut output, indent, self);
        output
    }
}

impl std::fmt::Display for Elements {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display(0))
    }
}

/// A Value in the tree
///
/// Like a simplified JSON data structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Tree(Elements),
}

impl Value {
    pub fn sub(element: Element) -> Self {
        Value::Tree(Elements(vec![element]))
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Str(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Str(s)
    }
}

impl From<Elements> for Value {
    fn from(elements: Elements) -> Self {
        Value::Tree(elements)
    }
}

/*
impl From<Element> for Value {
    fn from(b: Element) -> Self {
        Value::Tree(Box::new(b))
    }
}
*/

/// The status of a test run
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestRunStatus {
    Passed,
    Failed,
    Skipped,
}

/// A more detailed status of a test run
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestResults {
    /// Total inner tests
    pub nb_tests: usize,
    /// Total number of succesful tests
    pub nb_success: usize,
    /// Total number of failed tests
    pub nb_failed: usize,
    /// Total number of skipped tests
    pub nb_skipped: usize,
    /// Failures
    pub failures: Vec<String>,
    /// Duration for this overall tests
    pub duration: Duration,
}

impl TestResults {
    pub fn to_status(&self) -> TestRunStatus {
        if self.nb_tests == 0 {
            TestRunStatus::Skipped
        } else if self.nb_failed > 0 {
            TestRunStatus::Failed
        } else if self.nb_skipped == self.nb_tests {
            TestRunStatus::Skipped
        } else {
            TestRunStatus::Passed
        }
    }

    pub fn new() -> Self {
        TestResults {
            nb_tests: 0,
            nb_success: 0,
            nb_failed: 0,
            nb_skipped: 0,
            failures: Vec::new(),
            duration: Duration::new(0, 0),
        }
    }

    pub fn add_success(&mut self) {
        self.nb_tests += 1;
        self.nb_success += 1;
    }

    /*
    pub fn add_skipped(&mut self) {
        self.nb_tests += 1;
        self.nb_skipped += 1;
    }
    */

    pub fn add_failed(&mut self, reason: String) {
        self.nb_tests += 1;
        self.nb_failed += 1;
        self.failures.push(reason);
    }

    pub fn set_duration(&mut self, d: Duration) {
        self.duration = d
    }

    pub fn add_subtests(&mut self, sub_tests: &Self) {
        self.nb_tests += sub_tests.nb_tests;
        self.nb_success += sub_tests.nb_success;
        self.nb_failed += sub_tests.nb_failed;
        self.nb_skipped += sub_tests.nb_skipped;
        self.failures.extend_from_slice(&sub_tests.failures);
        self.duration += sub_tests.duration;
    }

    /*
    pub fn has_succeeded(&self) -> bool {
        self.nb_failed == 0
    }
    */
}
