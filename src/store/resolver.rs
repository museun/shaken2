use std::sync::Arc;
use tokio::sync::Mutex;

type BoxResolver = template::Resolver<Box<dyn template::TemplateStore + Send>>;

pub type Resolver = Arc<Mutex<BoxResolver>>;

pub fn new_resolver<S>(store: S) -> anyhow::Result<Resolver>
where
    S: template::TemplateStore + Send + 'static,
{
    template::Resolver::new(Box::new(store) as Box<dyn template::TemplateStore + Send>)
        .map_err(|err| anyhow::anyhow!("cannot create template resolver: {}", err))
        .map(Mutex::new)
        .map(Arc::new)
}
