use crate::handler::DynHandler;
use crate::{Handler, RespondableContext, Responder, Room, User};

use twitchchat::messages::Privmsg;

use futures::prelude::*;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Passive {
    pub message: Arc<Privmsg<'static>>,
    room_id: u64,
    user_id: u64,
}

impl Passive {
    pub fn new(message: Arc<Privmsg<'static>>) -> Option<Self> {
        // TODO log this
        let room_id = message.room_id()?;

        // TODO log this
        let user_id = message.user_id()?;

        Some(Self {
            message,
            room_id,
            user_id,
        })
    }
}

impl RespondableContext for Passive {
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
pub struct WrappedPassive<R> {
    pub inner: Arc<DynHandler<Passive, R>>,
    pub id: usize,
}

impl<R> std::fmt::Debug for WrappedPassive<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WrappedPassive")
            .field("id", &self.id)
            .finish()
    }
}

#[derive(Debug)]
pub struct PassiveList<R> {
    inner: Vec<WrappedPassive<R>>,
    id: usize,
    _phantom: std::marker::PhantomData<R>,
}

impl<R: Responder + Send + 'static> Default for PassiveList<R> {
    fn default() -> Self {
        let (inner, id, _phantom) = Default::default();
        Self {
            inner,
            id,
            _phantom,
        }
    }
}

impl<R: Responder + Send + 'static> PassiveList<R> {
    pub fn add<H, F>(&mut self, handler: H) -> usize
    where
        H: Handler<Passive, R, Fut = F>,
        F: Future<Output = anyhow::Result<()>>,
        F::Output: Send + 'static,
        F: Send + 'static,
    {
        let next = self.id + 1;
        let id = std::mem::replace(&mut self.id, next);
        self.inner.push(WrappedPassive {
            inner: Arc::new(move |state, resp| handler.call(state, resp)),
            id,
        });
        id
    }

    pub fn remove(&mut self, id: usize) -> Option<()> {
        let n = self.inner.iter().position(|s| s.id == id)?;
        self.inner.swap_remove(n);
        Some(())
    }

    pub fn iter(&self) -> PassiveIter<'_, R> {
        PassiveIter {
            inner: self,
            pos: 0,
        }
    }
}

pub struct PassiveIter<'a, R> {
    inner: &'a PassiveList<R>,
    pos: usize,
}

impl<'a, R> Iterator for PassiveIter<'a, R> {
    type Item = &'a WrappedPassive<R>;
    fn next(&mut self) -> Option<Self::Item> {
        let pos = self.pos;
        self.pos += 1;
        self.inner.inner.get(pos)
    }
}
