use super::*;

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
        let room = context.args.room();
        let user = context.args.user();
        log::trace!(
            "say {}.{}::{} to {} for {}",
            T::name(template::NameCasing::Original),
            T::namespace(template::NameCasing::Original),
            template.variant(template::NameCasing::Original),
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
        log::trace!(
            "reply {}.{}::{} to {} for {}",
            T::name(template::NameCasing::Original),
            T::namespace(template::NameCasing::Original),
            template.variant(template::NameCasing::Original),
            context.args.room(),
            context.args.user(),
        );
        self.inner.say(context, template)
    }

    fn action<'a, T, K>(&mut self, context: &'a Context<K>, template: &'a T) -> AnyhowFut<'a>
    where
        T: Template + Send + Sync,
        K: RespondableContext + Send + Sync + 'static,
    {
        log::trace!(
            "action {}.{}::{} to {} for {}",
            T::name(template::NameCasing::Original),
            T::namespace(template::NameCasing::Original),
            template.variant(template::NameCasing::Original),
            context.args.room(),
            context.args.user(),
        );
        self.inner.say(context, template)
    }
}
