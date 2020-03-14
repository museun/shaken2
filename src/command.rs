use crate::handler::DynHandler;
use crate::{Handler, RespondableContext, Responder, Room, User};

use std::future::Future;
use std::sync::Arc;

use twitchchat::messages::Privmsg;

#[derive(Debug, Clone)]
pub struct Command {
    pub head: Arc<str>,
    pub tail: Arc<[String]>,
    pub message: Arc<Privmsg<'static>>,

    room_id: u64,
    user_id: u64,
}

impl Command {
    const PREFIX: &'static str = "!";
    pub fn parse(message: Arc<Privmsg<'static>>) -> Option<Self> {
        let input = &*message.data.trim();
        if !input.starts_with(Self::PREFIX) {
            return None;
        }

        // TODO log this
        let room_id = message.room_id()?;

        // TODO log this
        let user_id = message.user_id()?;

        let mut iter = input[1..].split_terminator(' ');
        let head = iter.next()?;

        Some(Self {
            head: head.into(),
            tail: iter.map(ToString::to_string).collect(),
            message,

            room_id,
            user_id,
        })
    }

    pub fn join_tail(&self) -> Option<String> {
        Some(self.tail.iter().cloned().collect()) //
            .filter(|s: &String| !s.is_empty())
    }
}

impl RespondableContext for Command {
    fn room(&self) -> Room<'_> {
        Room {
            name: self.message.channel.clone(),
            id: self.room_id,
        }
    }

    fn user(&self) -> User<'_> {
        let name = self
            .message
            .display_name()
            .cloned()
            .unwrap_or_else(|| self.message.name.clone());
        User {
            name,
            id: self.user_id,
        }
    }
}

#[derive(Clone)]
pub struct WrappedCommand<R> {
    pub inner: Arc<DynHandler<Command, R>>,
    pub trigger: Arc<str>,
    pub id: usize,
}

impl<R> std::fmt::Debug for WrappedCommand<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WrappedCommand")
            .field("trigger", &self.trigger)
            .field("id", &self.id)
            .finish()
    }
}

#[derive(Debug)]
pub struct CommandMap<R> {
    inner: Vec<WrappedCommand<R>>,
    id: usize,
    responder: R,
}

impl<R> CommandMap<R>
where
    R: Responder + Send + 'static,
{
    pub fn new(responder: R) -> Self {
        let (inner, id) = Default::default();
        Self {
            inner,
            id,
            responder,
        }
    }

    pub fn add<H, F>(&mut self, trigger: impl ToString, handler: H) -> usize
    where
        H: Handler<Command, R, Fut = F>,
        F: Future<Output = anyhow::Result<()>>,
        F::Output: Send + 'static,
        F: Send + 'static,
    {
        let inner = Arc::new(move |state, responder| handler.call(state, responder));
        let trigger = trigger.to_string().as_str().into();

        let id = self.id;
        self.id += 1;
        self.inner.push(WrappedCommand { inner, trigger, id });
        id
    }

    pub fn responder(&self) -> R {
        self.responder.clone()
    }

    pub fn command_names(&self) -> impl Iterator<Item = Arc<str>> + '_ {
        self.inner.iter().map(|s| &s.trigger).map(Arc::clone)
    }

    pub fn remove(&mut self, id: usize) -> bool {
        match self.inner.iter().position(|s| s.id == id) {
            Some(pos) => {
                self.inner.swap_remove(pos);
                true
            }
            None => false,
        }
    }

    pub fn find(&self, trigger: &str) -> Vec<WrappedCommand<R>> {
        self.inner
            .iter()
            .filter(|s| &*s.trigger == trigger)
            .cloned()
            .collect()
    }
}
