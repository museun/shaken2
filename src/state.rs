use crate::util;

use std::any::{Any, TypeId};
use std::collections::HashMap;

use std::sync::Arc;
use tokio::sync::RwLock;

pub type StateRef = Arc<RwLock<State>>;

#[derive(Default, Debug)]
pub struct State {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl State {
    pub fn get<T>(&self) -> Option<&T>
    where
        T: 'static + Send + Sync,
    {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|item| item.downcast_ref::<T>())
    }

    pub fn expect_get<T>(&self) -> anyhow::Result<&T>
    where
        T: 'static + Send + Sync,
    {
        self.get().ok_or_else(|| {
            anyhow::anyhow!("cannot get: {}", util::type_name::<T>()) //
        })
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: 'static + Send + Sync,
    {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|item| item.downcast_mut::<T>())
    }

    pub fn expect_get_mut<T>(&mut self) -> anyhow::Result<&mut T>
    where
        T: 'static + Send + Sync,
    {
        self.get_mut().ok_or_else(|| {
            anyhow::anyhow!("cannot get: {}", util::type_name::<T>()) //
        })
    }

    pub fn insert<T>(&mut self, item: T) -> bool
    where
        T: 'static + Send + Sync,
    {
        self.map.insert(TypeId::of::<T>(), Box::new(item)).is_none()
    }
}
