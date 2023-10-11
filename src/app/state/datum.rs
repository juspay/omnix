/// Represent loading/refreshing state of UI data
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Datum<T> {
    #[default]
    Loading,
    Available {
        value: T,
        refreshing: bool,
    },
}

impl<T> Datum<T> {
    /// Get the inner value if available
    pub fn as_ref(&self) -> Option<&T> {
        match self {
            Datum::Loading => None,
            Datum::Available {
                value: x,
                refreshing: _,
            } => Some(x),
        }
    }

    pub fn is_loading_or_refreshing(&self) -> bool {
        matches!(
            self,
            Datum::Loading
                | Datum::Available {
                    value: _,
                    refreshing: true
                }
        )
    }

    pub fn current_value(&self) -> Option<&T> {
        match self {
            Datum::Loading => None,
            Datum::Available {
                value: x,
                refreshing: _,
            } => Some(x),
        }
    }

    pub fn set_value(&mut self, value: T) {
        *self = Datum::Available {
            value,
            refreshing: false,
        }
    }

    pub fn mark_refreshing(&mut self) {
        if let Datum::Available {
            value: _,
            refreshing,
        } = self
        {
            if *refreshing {
                panic!("Cannot refresh already refreshing data");
            }
            println!("🍎 refreshing...");
            *refreshing = true;
        }
    }
}
