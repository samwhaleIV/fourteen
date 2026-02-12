use super::VecPool;

use slotmap::{
    SlotMap,
    Key
};

pub struct SetBuffers<T,TKey: Key,const BUFFER_START_CAPACITY: usize> {
    pool: VecPool<T,BUFFER_START_CAPACITY>,
    in_use: SlotMap<TKey,Vec<T>>,
}

impl<T,TKey: Key,const BUFFER_START_CAPACITY: usize> SetBuffers<T,TKey,BUFFER_START_CAPACITY> {

    pub fn create(concurrent_buffer_capacity: usize) -> Self {
        Self {
            pool: VecPool::with_capacity(concurrent_buffer_capacity),
            in_use: SlotMap::with_capacity_and_key(concurrent_buffer_capacity)
        }
    }

    pub fn open_lease_with_entries(&mut self,entries: &[T]) -> TKey {
        todo!();
    }
    pub fn end_lease(&mut self,key: TKey) -> Option<()> {
        todo!();
    }
    pub fn get(&mut self,key: TKey) -> Option<&[T]> {
        self.in_use.get(key).map(|buffer|buffer.as_slice())
    }
    pub fn get_mut(&mut self,key: TKey) -> Option<&mut Vec<T>> {
        self.in_use.get_mut(key)
    }
    pub fn insert_active(&mut self,value: Vec<T>) -> TKey {
        todo!();
    }
    pub fn remove_active(&mut self,key: TKey) -> Option<Vec<T>> {
        todo!();
    }
}
