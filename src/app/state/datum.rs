/// Represent loading/refreshing state of UI data
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Datum<T> {
    #[default]
    Loading,
    Available(T),
    Refreshing(T),
}

impl<T> Datum<T> {
    /// Get the inner value if available
    pub fn as_ref(&self) -> Option<&T> {
        match self {
            Datum::Loading => None,
            Datum::Available(x) => Some(x),
            Datum::Refreshing(x) => Some(x),
        }
    }

    pub fn as_mut(&mut self) -> Option<&mut T> {
        match self {
            Datum::Loading => None,
            Datum::Available(x) => Some(x),
            Datum::Refreshing(x) => Some(x),
        }
    }
}
