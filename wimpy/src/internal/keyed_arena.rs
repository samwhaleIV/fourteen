use std::{
    collections::HashMap,
    hash::Hash,
};

use slotmap::{
    Key,
    SlotMap,
};

enum PoolLocation {
    Cache,
    Lease,
}

struct KeyData<TKey> {
    key: TKey,
    index: usize,
    location: PoolLocation,
}

struct SlotMapItem<TKey,TValue> {
    value: TValue,
    key_data: Option<KeyData<TKey>>,
}

type Pool<TReference> = Vec<TReference>;
struct PoolContainer<TKey,TReference> {
    initial_capacity: usize,
    map: HashMap<TKey,Pool<TReference>>
}

pub struct KeyedArena<TKey,TReference,TItem> where TReference: Key {
    slotmap: SlotMap<TReference,SlotMapItem<TKey,TItem>>,
    cache: PoolContainer<TKey,TReference>,
    leases: Pool<TReference>
}

pub enum KeyedArenaError<TKey,TReference> {
    ExpiredReference(TReference),
    KeylessReference(TReference),
    NoActiveLease(TReference,TKey),
    EmptyKeyedPool(TKey),
    MissingKeyedPool(TKey),
    PoolSwapMiss(TKey,usize),
    Generic
}

pub struct InitialCapacity {
    pub entries: usize,
    pub pool_count: usize,
    pub pool_size: usize,
    pub consecutive_leases: usize,
}

impl Default for InitialCapacity {
    fn default() -> Self {
        Self {
            entries: 256,
            pool_count: 16,
            pool_size: 16,
            consecutive_leases: 128
        }
    }
}

trait PoolContainerExtension<TKey,TReference> {
    fn get_pool_or_default_mut(&mut self,key: TKey) -> &mut Pool<TReference>;
    fn get_pool_mut(&mut self,key: &TKey) -> Option<&mut Pool<TReference>>;
}

impl<TKey,TReference> PoolContainerExtension<TKey,TReference> for PoolContainer<TKey,TReference> where TKey: Eq + Hash {
    fn get_pool_or_default_mut(&mut self,key: TKey) -> &mut Pool<TReference> {
        return self.map.entry(key).or_insert_with(||Vec::with_capacity(self.initial_capacity));
    }
    fn get_pool_mut(&mut self,key: &TKey) -> Option<&mut Pool<TReference>> {
        return self.map.get_mut(key);
    }
}

impl<TKey,TReference,TItem> Default for KeyedArena<TKey,TReference,TItem> where TReference: Key, TKey: Eq + Copy + Hash {
    fn default() -> Self {
        return Self::new(InitialCapacity::default());
    }
}

const MOVE_TO_CACHE: bool = true;
const MOVE_TO_LEASE: bool = false;

impl<TKey,TReference,TItem> KeyedArena<TKey,TReference,TItem> where TReference: Key, TKey: Eq + Copy + Hash {

    pub fn new(capacities: InitialCapacity) -> Self {
        return Self {
            slotmap: SlotMap::with_capacity_and_key(capacities.entries),
            cache: PoolContainer {
                initial_capacity: capacities.pool_size,
                map: HashMap::with_capacity(capacities.pool_count)
            },
            leases: Vec::with_capacity(capacities.consecutive_leases),
        }
    }

    pub fn insert_keyless(&mut self,item: TItem) -> TReference {
        return self.slotmap.insert(SlotMapItem { value: item, key_data: None });
    }

    fn insert_internal<const MOVE_TO_CACHE: bool>(&mut self,item: TItem,key: TKey) -> TReference {
        let (pool,location) = match MOVE_TO_CACHE {
            true => (self.cache.get_pool_or_default_mut(key),PoolLocation::Cache),
            false => (&mut self.leases,PoolLocation::Lease),
        };

        let reference = self.slotmap.insert(SlotMapItem {
            value: item,
            key_data: Some(KeyData {
                key,
                index: pool.len(),
                location
            })
        });

        pool.push(reference);

        return reference;
    }

    pub fn insert(&mut self,item: TItem,key: TKey) {
        self.insert_internal::<MOVE_TO_CACHE>(item,key);
    }

    pub fn insert_and_lease(&mut self,key: TKey,item: TItem) -> TReference {
        return self.insert_internal::<MOVE_TO_LEASE>(item,key);
    }

    pub fn remove(&mut self,reference: TReference) -> Result<TItem,KeyedArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.remove(reference) else {
            return Err(KeyedArenaError::ExpiredReference(reference));
        };
        let Some(KeyData{ key, index, location: lease_state }) = item.key_data else {
            return Ok(item.value);
        };

        let pool = match lease_state {
            PoolLocation::Cache => {
                let Some(pool) = self.cache.get_pool_mut(&key) else {
                    return Err(KeyedArenaError::MissingKeyedPool(key));
                };
                pool
            },
            PoolLocation::Lease => &mut self.leases,
        };

        pool.swap_remove(index);

        let Some(swapped_item_reference) = pool.get(index).cloned() else {
            return Err(KeyedArenaError::PoolSwapMiss(key,index));
        };
        let Some(swapped_item) = self.slotmap.get_mut(swapped_item_reference) else {
            return Err(KeyedArenaError::ExpiredReference(swapped_item_reference)); 
        };
        let Some(swapped_item_key_data) = &mut swapped_item.key_data else {
            return Err(KeyedArenaError::KeylessReference(swapped_item_reference));
        };
        swapped_item_key_data.index = index;

        return Ok(item.value);
    }

    pub fn get(&self,reference: TReference) -> Result<&TItem,KeyedArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.get(reference) else {
            return Err(KeyedArenaError::ExpiredReference(reference));
        };
        return Ok(&item.value);
    }

    pub fn get_mut(&mut self,reference: TReference) -> Result<&mut TItem,KeyedArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.get_mut(reference) else {
            return Err(KeyedArenaError::ExpiredReference(reference));
        };
        return Ok(&mut item.value);
    }

    fn pool_swap<const move_to_cache: bool>(&mut self,reference: TReference) -> Result<(),KeyedArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.get_mut(reference) else {
            return Err(KeyedArenaError::ExpiredReference(reference)); 
        };
        let Some(key_data) = &mut item.key_data else {
            return Err(KeyedArenaError::KeylessReference(reference));
        };

        let (origin,destination,lease_state) = {
            let Some(cache) = self.cache.get_pool_mut(&key_data.key) else {
                return Err(KeyedArenaError::MissingKeyedPool(key_data.key));
            };
            let leases = &mut self.leases;
            match MOVE_TO_CACHE {
                true => (leases,cache,PoolLocation::Cache),
                false => (cache,leases,PoolLocation::Lease),
            }
        };

        let origin_index = key_data.index;
        key_data.index = destination.len();
        key_data.location = lease_state;

        destination.push(reference);
        origin.swap_remove(key_data.index);

        let Some(swapped_item_reference) = origin.get(origin_index).cloned() else {
            return Err(KeyedArenaError::PoolSwapMiss(key_data.key,origin_index));
        };
        let Some(swapped_item) = self.slotmap.get_mut(swapped_item_reference) else {
            return Err(KeyedArenaError::ExpiredReference(swapped_item_reference)); 
        };
        let Some(swapped_item_key_data) = &mut swapped_item.key_data else {
            return Err(KeyedArenaError::KeylessReference(swapped_item_reference));
        };
        swapped_item_key_data.index = origin_index;
        
        return Ok(());
    }

    pub fn open_lease(&mut self,key: TKey) -> Result<TReference,KeyedArenaError<TKey,TReference>> {
        let Some(pool) = self.cache.get_pool_mut(&key) else {
            return Err(KeyedArenaError::MissingKeyedPool(key));
        };
        let Some(reference) = pool.last().cloned() else {
            return Err(KeyedArenaError::EmptyKeyedPool(key));
        };
        self.pool_swap::<{MOVE_TO_LEASE}>(reference)?;
        return Ok(reference);
    }

    pub fn end_lease(&mut self,reference: TReference) -> Result<(),KeyedArenaError<TKey,TReference>> {
        return self.pool_swap::<{MOVE_TO_CACHE}>(reference);
    }

    pub fn close_all_leases(&mut self) {
        loop {
            let Some(reference) = self.leases.pop() else {
                break;
            };
            let Some(item) = self.slotmap.get_mut(reference) else {
                continue;
            };
            let Some(key_data) = &mut item.key_data else {
                continue;
            };
            let Some(pool) = self.cache.get_pool_mut(&key_data.key) else {
                continue;
            };
            key_data.index = pool.len();
            pool.push(reference);
            key_data.location = PoolLocation::Cache;
        }
    }
}
