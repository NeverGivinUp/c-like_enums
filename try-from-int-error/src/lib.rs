use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Debug)]
pub struct TryFromIntError<T: Sized + Debug + Display>(pub T);

impl<T: Sized + Debug + Display> Display for TryFromIntError<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "Could not convert {}", self.0)
    }
}

impl<T: Sized + Debug + Display> std::error::Error for TryFromIntError<T> {}
