use crate::{Context, RespondableContext};

mod time;
pub use self::time::*;

pub mod serde;

mod ty;
pub use ty::*;

mod error;
pub use error::*;

/// Check the specific configuration list to see if context room is on it
///
/// ```ignore
/// // if the room is on the whitelist, this will return Ok
/// check_config(&mut context, |config: Shakespeare| config.whitelist)?;
/// ```
// TODO rename this function
// or do something different
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
    context.args.room().check_list(config.into_iter())
}
