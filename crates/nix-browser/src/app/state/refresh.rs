use std::fmt::Display;

/// Represents an user request to update some thing (a dioxus Signal)
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Refresh {
    idx: usize,
}

impl Display for Refresh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.idx)
    }
}

impl Refresh {
    pub fn request_refresh(&mut self) {
        tracing::info!("🔄 Requesting refresh of a signal");
        self.idx += 1;
    }
}
