use serde::{Deserialize, Serialize};

/// Health report
///
/// If you just want the binary indicator, use `Report<NoDetails>` (see
/// [NoDetails]). If you want the report with details regarding the problem, use
/// `Report<WithDetails>` (see [WithDetails]).
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub enum Report<T> {
    /// Green means everything is fine
    Green,
    /// Red means something is wrong. `T` holds information about what's wrong.
    Red(T),
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct NoDetails;

impl<R> Report<R> {
    pub fn is_green(&self) -> bool {
        match self {
            Report::Green => true,
            Report::Red(_) => false,
        }
    }

    pub fn is_red(&self) -> bool {
        !self.is_green()
    }
}

/// Details regarding a failed report
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize, Clone)]
pub struct WithDetails {
    /// A short message describing the problem
    pub msg: String,
    /// A suggestion for how to fix the problem
    pub suggestion: String,
} // TODO: Should this be Markdown?

impl Report<WithDetails> {
    /// Return the report without the details
    pub fn without_details(&self) -> Report<NoDetails> {
        match self {
            Report::Green => Report::Green,
            Report::Red(_) => Report::Red(NoDetails),
        }
    }
    /// Return the problem details if there is one.
    pub fn get_red_details(&self) -> Option<WithDetails> {
        match self {
            Report::Green => None,
            Report::Red(details) => Some(details.clone()),
        }
    }
}
