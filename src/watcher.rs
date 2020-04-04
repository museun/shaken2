use futures::prelude::*;
use notify::Watcher as _;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, watch, Notify};

pub struct Watcher {
    rx: mpsc::Receiver<notify::event::Event>,
    notify: Arc<Notify>,
    watcher: notify::RecommendedWatcher,
    watched: HashMap<PathBuf, mpsc::Sender<PathBuf>>,
}

impl Watcher {
    pub fn new() -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel(32);
        let notify = Arc::new(Notify::new());

        let watcher = notify::RecommendedWatcher::new_immediate({
            let notify = notify.clone();
            // mutex because the closure is an Fn and we should probably
            // synchronize borrows
            let tx = Mutex::new(tx);
            move |ev| {
                if let Ok(ev) = ev {
                    if tx.lock().unwrap().try_send(ev).is_ok() {
                        return;
                    }
                }
                notify.notify();
            }
        })?;

        Ok(Self {
            rx,
            notify,
            watcher,
            watched: HashMap::new(),
        })
    }

    pub async fn watch_file<C: Clone>(
        &mut self,
        file: impl AsRef<Path>,
        item: C,
    ) -> anyhow::Result<watch::Receiver<C>>
    where
        for<'de> C: serde::Deserialize<'de> + Send + Sync + 'static,
    {
        let (watch_tx, watch_rx) = watch::channel::<C>(item);
        self.watcher
            .watch(&file, notify::RecursiveMode::NonRecursive)?;

        let (tx, mut rx) = tokio::sync::mpsc::channel::<PathBuf>(1);

        // detach the task because the channel controls when it dies
        // so dropping the sender from the hashmap will end this task
        tokio::task::spawn(async move {
            while let Some(path) = rx.next().await {
                let data = match tokio::fs::read_to_string(&path).await {
                    Ok(data) => data,
                    Err(err) => {
                        log::warn!("cannot read '{}': {}", path.display(), err);
                        break;
                    }
                };

                let item = match toml::from_str(&data) {
                    Ok(item) => item,
                    Err(err) => {
                        log::warn!("cannot deserialize '{}': {}", path.display(), err);
                        break;
                    }
                };

                if watch_tx.broadcast(item).is_err() {
                    break;
                }
            }
        });

        self.watched
            .insert(tokio::fs::canonicalize(file).await?, tx);

        Ok(watch_rx)
    }

    pub fn abort_handle(&self) -> Arc<Notify> {
        Arc::clone(&self.notify)
    }

    pub async fn run_to_completion(mut self) -> anyhow::Result<()> {
        let delay = std::time::Duration::from_secs(1);
        tokio::pin! {
            let notify = self.notify;
            let rx = self.rx;
            let delay_tick = tokio::time::interval(delay);
        }

        let mut queue = DebounceQueue::<PathBuf>::new(delay);

        loop {
            tokio::select! {
                _ = notify.notified() => { break }
                Some(instant) = delay_tick.next() => {
                    if let Some(item) = queue.force(instant) {
                        if let Ok(canon) = tokio::fs::canonicalize(item).await {
                            if let Some(tx) = self.watched.get_mut(&canon) {
                                tx.send(canon).await.expect("send item");
                            } else {
                                log::error!("path not found :(");
                            }
                        }
                    }
                }
                Some(ev) = rx.next() => {
                    if let notify::event::EventKind::Modify(notify::event::ModifyKind::Any) = ev.kind {
                        let mut ev = ev;
                        // if notify messes up, this panics. but thats not our problem
                        queue.push(ev.paths.swap_remove(0));
                    }
                }
                else => { break }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct DebounceQueue<T> {
    freq: std::time::Duration,
    last: tokio::time::Instant,
    queue: Vec<T>,
}

impl<T> DebounceQueue<T> {
    fn new(freq: std::time::Duration) -> Self {
        Self {
            freq,
            last: tokio::time::Instant::now(),
            queue: vec![],
        }
    }
}

impl<T> DebounceQueue<T> {
    fn force(&mut self, comp: tokio::time::Instant) -> Option<T> {
        if comp.saturating_duration_since(self.last) > self.freq {
            return self.queue.drain(..).last();
        }
        None
    }

    fn push(&mut self, item: T) {
        let now = tokio::time::Instant::now();
        self.queue.push(item);
        self.last = now;
    }
}
