use std::{
    collections::HashSet,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

use anyhow::bail;

#[doc(hidden)]
pub trait UniqueIdName {
    fn id(&self) -> u32;
    fn name(&self) -> &str;
}

struct ByName<T>(Arc<T>);

impl<T: UniqueIdName> Hash for ByName<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.name().hash(state);
    }
}

impl<T: UniqueIdName> PartialEq for ByName<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.name() == other.0.name()
    }
}

impl<T: UniqueIdName> Eq for ByName<T> {}

impl<T> Clone for ByName<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct ById<T>(Arc<T>);

impl<T: UniqueIdName> Hash for ById<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.id().hash(state);
    }
}

impl<T: UniqueIdName> PartialEq for ById<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.id() == other.0.id()
    }
}

impl<T: UniqueIdName> Eq for ById<T> {}

impl<T> Clone for ById<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// Storage for entities with unique names and ids,
/// allowing to faster search by both values.
///
/// Primarely used for storing indices inside space metadata.
pub struct UniqueIdNameMap<T> {
    by_name: HashSet<ByName<T>>,
    by_id: HashSet<ById<T>>,
}

impl<T> UniqueIdNameMap<T> {
    pub(crate) fn new() -> Self {
        Self::with_capacity(1)
    }

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            by_name: HashSet::with_capacity(capacity),
            by_id: HashSet::with_capacity(capacity),
        }
    }
}

impl<T: UniqueIdName> UniqueIdNameMap<T> {
    pub(crate) fn insert(&mut self, value: T) -> Result<Option<Arc<T>>, anyhow::Error> {
        let value = Arc::new(value);
        let old_name_value = self.by_name.replace(ByName(value.clone()));
        let old_id_value = self.by_id.replace(ById(value));
        match (&old_name_value, &old_id_value) {
            (Some(left), Some(right)) if !Arc::ptr_eq(&left.0, &right.0) => {
                bail!(
                    "New value with name '{}' and id '{}' replaced 2 different old values",
                    left.0.name(),
                    right.0.id()
                )
            }
            (Some(left), None) => {
                bail!(
                    "New value with name '{}' replaced only by name",
                    left.0.name(),
                )
            }
            (None, Some(right)) => {
                bail!("New value with id '{}' replaced only by id", right.0.name(),)
            }
            _ => {}
        }
        Ok(old_name_value.map(|x| x.0))
    }
}

impl<T> Default for UniqueIdNameMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for UniqueIdNameMap<T> {
    fn clone(&self) -> Self {
        Self {
            by_name: self.by_name.clone(),
            by_id: self.by_id.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for UniqueIdNameMap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.by_id.iter().map(|x| (&*x.0)))
            .finish()
    }
}

impl<T: UniqueIdName> UniqueIdNameMap<T> {
    pub(crate) fn try_from_iter<I>(iter: I) -> Result<Self, anyhow::Error>
    where
        I: IntoIterator<Item = T>,
    {
        let iter = iter.into_iter();
        let size_hint = if let (start, Some(end)) = iter.size_hint() {
            end.saturating_sub(start)
        } else {
            1
        };
        let mut map = Self::with_capacity(size_hint);
        for x in iter {
            let _ = map.insert(x)?;
        }
        Ok(map)
    }
}
