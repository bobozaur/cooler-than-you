use thiserror::Error as ThisError;

/// Trait for implementing something similar to
/// <https://docs.rs/itertools/latest/itertools/trait.Itertools.html#method.exactly_one>
/// but in a simpler form.
pub trait ExactlyOneIter: Iterator {
    fn exactly_one(&mut self) -> Result<Self::Item, ExactlyOneError> {
        match (self.next(), self.next()) {
            (None, _) => Err(ExactlyOneError::Zero),
            (Some(item), None) => Ok(item),
            (Some(_), _) => Err(ExactlyOneError::MoreThanTwo),
        }
    }
}

impl<T> ExactlyOneIter for T where T: Iterator {}

#[derive(Debug, ThisError)]
pub enum ExactlyOneError {
    #[error("zero items found")]
    Zero,
    #[error("more than two items found")]
    MoreThanTwo,
}
