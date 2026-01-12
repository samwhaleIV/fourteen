use std::{
    hash::Hash,
    marker::PhantomData
};

use slotmap::{
    Key,
    SlotMap,
};

use crate::internal::keyed_pools::{
    KeyedPools,
    MoveToCache,
    MoveToLease,
    PoolOriginDestination,
    PoolSelector,
    PoolTarget
};

struct KeyData<TKey> {
    key: TKey,
    index: usize,
    pool_target: PoolTarget,
}

struct SlotMapItem<TKey,TValue> {
    value: TValue,
    key_data: Option<KeyData<TKey>>,
}

pub struct CachesArena<TKey,TReference,TItem,TConfig> where
    TReference: Key 
{
    slotmap: SlotMap<TReference,SlotMapItem<TKey,TItem>>,
    pools: KeyedPools<TKey,TReference,TConfig>,
    phantom_config: std::marker::PhantomData<TConfig>
}

pub enum CachesArenaError<TKey,TReference> {
    ExpiredReference(TReference),
    KeylessReference(TReference),
    NoActiveLease(TReference,TKey),
    EmptyKeyedPool(TKey),
    MissingKeyedPool(TKey),
    PoolSwapMiss(TKey,usize),
    Generic
}

pub trait CapacityConfig {
    const ENTRIES: usize;
    const POOL_COUNT: usize;
    const POOL_SIZE: usize;
    const LEASES: usize;
}

pub struct DefaultCapacityConfig;
impl CapacityConfig for DefaultCapacityConfig {
    const ENTRIES: usize = 128;
    const LEASES: usize = 128;
    const POOL_COUNT: usize = 16;
    const POOL_SIZE: usize = 16;
}

impl<TKey,TReference,TItem,TConfig> Default for CachesArena<TKey,TReference,TItem,TConfig> where
    TReference: Key,
    TKey: Eq + Copy + Hash,
    TConfig: CapacityConfig
{
    fn default() -> Self {
        return Self::new();
    }
}

impl<TKey,TReference,TItem,TConfig> CachesArena<TKey,TReference,TItem,TConfig> where
    TReference: Key,
    TKey: Eq + Copy + Hash,
    TConfig: CapacityConfig 
{

    pub fn new() -> Self {
        return Self {
            slotmap: SlotMap::with_capacity_and_key(TConfig::ENTRIES),
            pools: KeyedPools::new(),
            phantom_config: PhantomData
        }
    }

    pub fn insert_keyless(&mut self,item: TItem) -> TReference {
        return self.slotmap.insert(SlotMapItem { value: item, key_data: None });
    }

    pub fn insert(&mut self,item: TItem,key: TKey) {
        let cache_pool = self.pools.get_or_create_cache_mut(key);
        let pool_target = PoolTarget::Cache;

        let reference = self.slotmap.insert(SlotMapItem {
            value: item,
            key_data: Some(KeyData {
                key,
                index: cache_pool.len(),
                pool_target
            })
        });

        cache_pool.push(reference);
    }

    pub fn insert_with_lease(&mut self,key: TKey,item: TItem) -> TReference {
        self.pools.ensure_cache(key);

        let lease_pool = self.pools.get_lease_pool_mut();
        let pool_target = PoolTarget::Cache;

        let reference = self.slotmap.insert(SlotMapItem {
            value: item,
            key_data: Some(KeyData {
                key,
                index: lease_pool.len(),
                pool_target
            })
        });

        lease_pool.push(reference);

        return reference;
    }

    pub fn remove(&mut self,reference: TReference) -> Result<TItem,CachesArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.remove(reference) else {
            return Err(CachesArenaError::ExpiredReference(reference));
        };
        let Some(KeyData{ key, index, pool_target: lease_state }) = item.key_data else {
            return Ok(item.value);
        };

        let pool = match lease_state {
            PoolTarget::Cache => {
                let Some(cache) = self.pools.get_cache_mut(&key) else {
                    return Err(CachesArenaError::MissingKeyedPool(key));
                };
                cache
            },
            PoolTarget::Lease => self.pools.get_lease_pool_mut(),
        };

        pool.swap_remove(index);

        let Some(swapped_item_reference) = pool.get(index).cloned() else {
            return Err(CachesArenaError::PoolSwapMiss(key,index));
        };
        let Some(swapped_item) = self.slotmap.get_mut(swapped_item_reference) else {
            return Err(CachesArenaError::ExpiredReference(swapped_item_reference)); 
        };
        let Some(swapped_item_key_data) = &mut swapped_item.key_data else {
            return Err(CachesArenaError::KeylessReference(swapped_item_reference));
        };
        swapped_item_key_data.index = index;

        return Ok(item.value);
    }

    pub fn get(&self,reference: TReference) -> Result<&TItem,CachesArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.get(reference) else {
            return Err(CachesArenaError::ExpiredReference(reference));
        };
        return Ok(&item.value);
    }

    pub fn get_mut(&mut self,reference: TReference) -> Result<&mut TItem,CachesArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.get_mut(reference) else {
            return Err(CachesArenaError::ExpiredReference(reference));
        };
        return Ok(&mut item.value);
    }

    fn pool_swap<PoolStrategy: PoolSelector<TKey,TReference>>(&mut self,reference: TReference) -> Result<(),CachesArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.get_mut(reference) else {
            return Err(CachesArenaError::ExpiredReference(reference)); 
        };
        let Some(key_data) = &mut item.key_data else {
            return Err(CachesArenaError::KeylessReference(reference));
        };

        let PoolOriginDestination { origin, destination, target } = PoolStrategy::order(match self.pools.get_cache_and_lease_mut(&key_data.key) {
            Some(value) => value,
            None => return Err(CachesArenaError::MissingKeyedPool(key_data.key)),
        });

        let origin_index = key_data.index;
        key_data.index = destination.len();
        key_data.pool_target = target;

        destination.push(reference);
        origin.swap_remove(key_data.index);

        let Some(swapped_item_reference) = origin.get(origin_index).cloned() else {
            return Err(CachesArenaError::PoolSwapMiss(key_data.key,origin_index));
        };
        let Some(swapped_item) = self.slotmap.get_mut(swapped_item_reference) else {
            return Err(CachesArenaError::ExpiredReference(swapped_item_reference)); 
        };
        let Some(swapped_item_key_data) = &mut swapped_item.key_data else {
            return Err(CachesArenaError::KeylessReference(swapped_item_reference));
        };
        swapped_item_key_data.index = origin_index;
        
        return Ok(());
    }

    pub fn start_lease(&mut self,key: TKey) -> Result<TReference,CachesArenaError<TKey,TReference>> {
        let Some(cache) = self.pools.get_cache_mut(&key) else {
            return Err(CachesArenaError::MissingKeyedPool(key));
        };
        let Some(reference) = cache.last().cloned() else {
            return Err(CachesArenaError::EmptyKeyedPool(key));
        };
        self.pool_swap::<MoveToLease>(reference)?;
        return Ok(reference);
    }

    pub fn end_lease(&mut self,reference: TReference) -> Result<(),CachesArenaError<TKey,TReference>> {
        return self.pool_swap::<MoveToCache>(reference);
    }

    pub fn end_all_leases(&mut self) {
        loop {
            let Some(reference) = self.pools.pop_lease() else {
                break;
            };
            let Some(item) = self.slotmap.get_mut(reference) else {
                continue;
            };
            let Some(key_data) = &mut item.key_data else {
                continue;
            };
            let Some(cache) = self.pools.get_cache_mut(&key_data.key) else {
                continue;
            };
            key_data.index = cache.len();
            cache.push(reference);
            key_data.pool_target = PoolTarget::Cache;
        }
    }
}
