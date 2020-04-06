use super::*;

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
            let room = context.args.room();
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
            let room = context.args.room();
            let user = context.args.user();

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
            let target = context.args.room();
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
            .resolve(
                T::namespace(Default::default()),
                template.variant(Default::default()),
            )
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "cannot resolve template for '{}' {}::{}",
                    T::name(template::NameCasing::Original),
                    T::namespace(template::NameCasing::Original),
                    template.variant(template::NameCasing::Original),
                )
            })
    }

    pub(crate) fn apply_template<T: Template>(template: &T, data: &str) -> anyhow::Result<String> {
        template.apply(data).ok_or_else(|| {
            anyhow::anyhow!(
                "invalid template for '{}' {}::{}",
                T::name(template::NameCasing::Original),
                T::namespace(template::NameCasing::Original),
                template.variant(template::NameCasing::Original)
            )
        })
    }
}
