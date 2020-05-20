use std::collections::{HashMap, HashSet};

pub trait Collection {
    //pub fn length
}

impl<T> Collection for &[T] {}

impl<T> Collection for Vec<T> {}

impl<T, H> Collection for HashSet<T, H> {}

impl<K, V, H> Collection for HashMap<K, V, H> {}
