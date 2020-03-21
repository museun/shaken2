/// Error type that isn't an error, but isn't a successful value
///
/// When this is returned by a handler, the 'error' won't be logged or treated
/// like an error
#[derive(Debug)]
pub struct DontCareSigil {}

impl std::fmt::Display for DontCareSigil {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}
impl std::error::Error for DontCareSigil {}

/// Trait to create an `Err(DontCareSigil)` from a None/Err
pub trait DontCare<T> {
    fn dont_care(self) -> anyhow::Result<T>;
}

impl<T> DontCare<T> for Option<T> {
    fn dont_care(self) -> anyhow::Result<T> {
        self.ok_or_else(|| DontCareSigil {}.into())
    }
}

/// Helper function for creating an `Err(DontCareSigil)`
pub fn dont_care() -> anyhow::Result<()> {
    Err(DontCareSigil {}.into())
}
