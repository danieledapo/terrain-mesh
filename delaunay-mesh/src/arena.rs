use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct Arena<T> {
    data: Vec<T>,
}

#[derive(Debug)]
pub struct ArenaId<Tag> {
    ix: usize,
    tag: std::marker::PhantomData<Tag>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena { data: vec![] }
    }

    pub fn push(&mut self, v: T) -> ArenaId<T> {
        self.data.push(v);
        ArenaId::new(self.data.len() - 1)
    }

    pub fn replace(&mut self, id: ArenaId<T>, mut v: T) -> T {
        std::mem::swap(&mut self[id], &mut v);
        v
    }

    pub fn get(&self, id: ArenaId<T>) -> Option<&T> {
        self.data.get(id.ix)
    }

    pub fn get_mut(&mut self, id: ArenaId<T>) -> Option<&mut T> {
        self.data.get_mut(id.ix)
    }
}

impl<T> Index<ArenaId<T>> for Arena<T> {
    type Output = T;

    fn index(&self, ix: ArenaId<T>) -> &Self::Output {
        self.get(ix).unwrap()
    }
}

impl<T> IndexMut<ArenaId<T>> for Arena<T> {
    fn index_mut(&mut self, ix: ArenaId<T>) -> &mut T {
        self.get_mut(ix).unwrap()
    }
}

impl<Tag> ArenaId<Tag> {
    fn new(ix: usize) -> Self {
        ArenaId {
            ix,
            tag: PhantomData,
        }
    }
}

impl<T> Copy for ArenaId<T> {}
impl<T> Clone for ArenaId<T> {
    fn clone(&self) -> Self {
        ArenaId {
            ix: self.ix,
            tag: self.tag,
        }
    }
}
