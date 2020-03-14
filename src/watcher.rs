use notify::Watcher as _;
use std::path::Path;

/// tokio broadcast file watcher
pub struct Watcher {
    watcher: notify::RecommendedWatcher,
    rx: crossbeam_channel::Receiver<Result<notify::event::Event, notify::Error>>,
    watched:
        futures::stream::FuturesUnordered<futures::future::BoxFuture<'static, anyhow::Result<()>>>,
}

impl Watcher {
    /// create a new watcher
    pub fn new() -> anyhow::Result<Self> {
        let (tx, rx) = crossbeam_channel::bounded(32);
        let watcher = notify::watcher(tx, std::time::Duration::from_secs(2))?;
        Ok(Self {
            watcher,
            rx,
            watched: Default::default(),
        })
    }

    /// watch this file, provided the initial item
    pub fn watch<C: Clone>(
        &mut self,
        file: impl AsRef<Path>,
        item: C,
    ) -> anyhow::Result<tokio::sync::watch::Receiver<C>>
    where
        for<'de> C: serde::Deserialize<'de> + Send + Sync + 'static,
    {
        let (watch_tx, watch_rx) = tokio::sync::watch::channel(item);
        self.watcher
            .watch(file, notify::RecursiveMode::NonRecursive)?;

        let rx = self.rx.clone();
        let fut = tokio::task::spawn_blocking(move || {
            for event in rx.into_iter().flatten() {
                use notify::event::{DataChange, Event, EventKind, ModifyKind};

                let Event { kind, paths, .. } = event;
                if paths.is_empty() {
                    continue;
                }

                match kind {
                    EventKind::Modify(ModifyKind::Data(DataChange::Content))
                    | EventKind::Modify(ModifyKind::Any) => {}
                    _ => continue,
                }

                if let Some(item) = std::fs::read_to_string(&paths[0])
                    .ok()
                    .and_then(|data| toml::from_str(&data).ok())
                {
                    let _ = watch_tx.broadcast(item);
                }
            }
        });

        use futures::prelude::*;
        self.watched
            .push(fut.map_err(Into::into).map_ok(|_| ()).boxed());
        Ok(watch_rx)
    }

    pub async fn run_to_completion(self) {
        use futures::stream::StreamExt as _;
        self.watched
            .for_each(|res| async move {
                if let Err(err) = res {
                    log::error!("runner ran into an error: {}", err)
                }
            })
            .await;
    }
}
