use super::*;

#[derive(Debug, Copy, Clone, Default)]
pub struct NullResponder {}

impl Responder for NullResponder {
    fn say<'a, T, K>(&mut self, _: &'a Context<K>, _: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static,
    {
        async move { Ok(()) }.boxed()
    }

    fn reply<'a, T, K>(&mut self, _: &'a Context<K>, _: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static,
    {
        async move { Ok(()) }.boxed()
    }

    fn action<'a, T, K>(&mut self, _: &'a Context<K>, _: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static,
    {
        async move { Ok(()) }.boxed()
    }
}
