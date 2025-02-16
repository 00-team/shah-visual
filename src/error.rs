use shah::ShahError;

#[derive(Debug)]
pub enum ShahVisualError {
    Shah(ShahError),
    NotShahDb,
}

// impl From<ShahError> for ShahVisualError {
//     fn from(value: ShahError) -> Self {
//         ShahVisualError::Shah(value)
//     }
// }

// impl From<std::io::Error> for ShahVisualError {
//     fn from(value: std::io::Error) -> Self {
//         ShahVisualError::Shah(ShahError::from(value))
//     }
// }

impl<T: Into<ShahError>> From<T> for ShahVisualError {
    fn from(value: T) -> Self {
        Self::Shah(value.into())
    }
}

/// # Shah Visual Result
///  
/// this is a custom Result Type
/// wrapping the Ok value with ShahVisualError
/// as its Err
pub type Result<T> = core::result::Result<T, ShahVisualError>;
