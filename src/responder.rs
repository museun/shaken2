use crate::{Room, User};

use futures::prelude::*;
use template::Template;
use twitchchat::Writer;

use crate::{Context, Resolver};

pub trait RespondableContext {
    fn room(&self) -> Room<'_>;
    fn user(&self) -> User<'_>;
}

pub type AnyhowFut<'a> = future::BoxFuture<'a, anyhow::Result<()>>;

pub trait Responder: Clone {
    fn say<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static;

    fn reply<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static;

    fn action<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static;
}

#[derive(Clone)]
pub struct LoggingResponder<R: Responder> {
    inner: R,
}

impl<R: Responder> LoggingResponder<R> {
    pub fn new(responder: R) -> Self {
        Self { inner: responder }
    }
}

impl<R: Responder> Responder for LoggingResponder<R> {
    fn say<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static,
    {
        let args = context.args();
        let room = args.room();
        let user = args.user();
        log::trace!(
            "say {}.{}::{} to {} for {}",
            T::name(),
            T::namespace(),
            template.variant(),
            room,
            user
        );
        self.inner.say(context, template)
    }

    fn reply<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static,
    {
        let args = context.args();
        log::trace!(
            "reply {}.{}::{} to {} for {}",
            T::name(),
            T::namespace(),
            template.variant(),
            args.room(),
            args.user(),
        );
        self.inner.say(context, template)
    }

    fn action<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static,
    {
        let args = context.args();
        log::trace!(
            "action {}.{}::{} to {} for {}",
            T::name(),
            T::namespace(),
            template.variant(),
            args.room(),
            args.user(),
        );
        self.inner.say(context, template)
    }
}

#[derive(Clone)]
pub struct WriterResponder {
    writer: Writer,
    resolver: Resolver,
}

impl WriterResponder {
    pub fn new(writer: Writer, resolver: Resolver) -> Self {
        Self { writer, resolver }
    }
}

impl Responder for WriterResponder {
    fn say<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static,
    {
        let resolver = self.resolver.clone();
        let mut writer = self.writer.clone();

        async move {
            let room = context.args().room();
            let data = Self::resolve_template(resolver, template).await?;
            let resp = Self::apply_template(template, &data)?;
            writer.privmsg(&room.name, &resp).await?;
            Ok(())
        }
        .boxed()
    }

    fn reply<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static,
    {
        let resolver = self.resolver.clone();
        let mut writer = self.writer.clone();

        async move {
            let room = context.args().room();
            let user = context.args().user();

            let data = Self::resolve_template(resolver, template).await?;
            let resp = Self::apply_template(template, &data)?;
            writer
                .privmsg(&room.name, &format!("@{}: {}", user.name, &resp))
                .await?;
            Ok(())
        }
        .boxed()
    }

    fn action<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static,
    {
        let resolver = self.resolver.clone();
        let mut writer = self.writer.clone();

        async move {
            let target = context.args().room();
            let data = Self::resolve_template(resolver, template).await?;
            let resp = Self::apply_template(template, &data)?;
            writer.me(&target.name, &resp).await?;
            Ok(())
        }
        .boxed()
    }
}

impl WriterResponder {
    pub(crate) async fn resolve_template<T: Template>(
        resolver: Resolver,
        template: &T,
    ) -> anyhow::Result<String> {
        resolver
            .lock()
            .await
            .resolve(T::namespace(), template.variant())
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "cannot resolve template for '{}' {}::{}",
                    T::name(),
                    T::namespace(),
                    template.variant(),
                )
            })
    }

    pub(crate) fn apply_template<T: Template>(template: &T, data: &str) -> anyhow::Result<String> {
        template.apply(data).ok_or_else(|| {
            anyhow::anyhow!(
                "invalid template for '{}' {}::{}",
                T::name(),
                T::namespace(),
                template.variant()
            )
        })
    }
}

// pub(crate) async fn get_user(&self, target: u64) -> Result<String, ResponderError> {
//     self.tracker
//         .users
//         .get(target)
//         .await
//         .ok_or_else(|| ResponderError::InvalidRoom(target))
// }

// pub(crate) async fn get_room(&self, target: u64) -> Result<String, ResponderError> {
//     self.tracker
//         .rooms
//         .get(target)
//         .await
//         .ok_or_else(|| ResponderError::InvalidRoom(target))
// }
