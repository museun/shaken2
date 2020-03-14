use crate::{Context, Responder, WriterResponder};

use futures::prelude::*;

type Result = anyhow::Result<()>;

pub type BoxFuture = futures::future::BoxFuture<'static, Result>;

pub type DynHandler<Args, R = WriterResponder> =
    dyn Handler<Args, R, Fut = BoxFuture> + Send + 'static;

pub trait Handler<Args, R>
where
    Self: Send + 'static,
    Args: Send + 'static,
    R: Send + 'static,
{
    type Fut: Future<Output = Result> + Send + 'static;
    fn call(&self, state: Context<Args>, responder: R) -> Self::Fut;
}

impl<F, Fut, Args, R> Handler<Args, R> for F
where
    F: Fn(Context<Args>, R) -> Fut + Send + 'static,
    Fut: Future<Output = Result> + Send + 'static,
    Args: Send + 'static,
    R: Responder + Send + 'static,
{
    type Fut = BoxFuture;
    fn call(&self, state: Context<Args>, responder: R) -> Self::Fut {
        (self)(state, responder).boxed()
    }
}
