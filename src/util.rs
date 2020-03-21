use crate::{Context, RespondableContext};

/// Get the current timestamp (time since the UNIX epoch) in milliseconds
pub fn timestamp_ms() -> u64 {
    get_timestamp().as_millis() as _
}

/// Get the current timestamp (time since the UNIX epoch) in seconds
pub fn timestamp() -> u64 {
    get_timestamp().as_secs() as _
}

fn get_timestamp() -> std::time::Duration {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
}

/// Get a short representation of the provided type
pub fn type_name<T>() -> &'static str {
    reduce_type_name(std::any::type_name::<T>())
}

/// Tries to reduce a complex type name down to its base type
pub fn reduce_type_name(mut input: &str) -> &str {
    // this is .. totally not something you should do
    fn trim_type(input: &str) -> &str {
        let mut n = input.len();
        let left = input
            .chars()
            .take_while(|&c| {
                if c == '<' {
                    n -= 1;
                }
                !c.is_ascii_uppercase()
            })
            .count();
        &input[left..n]
    }

    let original = input;
    loop {
        let start = input.len();
        input = trim_type(input);
        if input.contains('<') {
            input = trim_type(&input[1..]);
        }
        match input.len() {
            0 => break original,
            d if d == start => break input,
            _ => {}
        }
    }
}

pub trait TypeName {
    fn type_name(&self) -> &'static str {
        #[allow(dead_code)]
        fn ty<T>(_ignored: &T) -> &'static str {
            reduce_type_name(type_name::<T>())
        }
        ty(&self)
    }
}

impl<T> TypeName for T {}

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

pub trait DontCare<T> {
    fn dont_care(self) -> anyhow::Result<T>;
}

impl<T> DontCare<T> for Option<T> {
    fn dont_care(self) -> anyhow::Result<T> {
        self.ok_or_else(|| DontCareSigil {}.into())
    }
}

/// Helper function for creating an Err(DontCareSigil)
pub fn dont_care() -> anyhow::Result<()> {
    Err(DontCareSigil {}.into())
}

/// Check the specific configuration list to see if context room is on it
///
/// ```ignore
/// // if the room is on the whitelist, this will return Ok
/// check_config(&mut context, |config: Shakespeare| config.whitelist)?;
/// ```
// TODO rename this function
pub async fn check_config<T, F, C, I>(context: &mut Context<T>, func: F) -> anyhow::Result<()>
where
    T: RespondableContext,
    C: Clone + Send + Sync + 'static,
    F: Fn(C) -> I,
    I: IntoIterator<Item = String>,
{
    use crate::name_and_id::NameAndId as _;
    let state = context.state().await;
    let config = state.expect_get::<C>()?;
    let config = func(config.clone());
    context.args().room().check_list(config.into_iter())
}
