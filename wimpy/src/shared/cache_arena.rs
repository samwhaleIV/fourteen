use std::{
    hash::Hash,
    marker::PhantomData,
    fmt::Debug
};

use slotmap::{
    Key,
    SlotMap,
};

use crate::shared::keyed_pools::{
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

#[derive(Debug)]
pub enum CacheArenaError<TKey,TReference> {
    ExpiredReference(TReference),
    KeylessReference(TReference),
    EmptyKeyedPool(TKey),
    MissingKeyedPool(TKey),
    NotInLease(TReference,TKey),
    NotInCache(TReference,TKey),
    PoolSwapAliasing(TKey,usize),
}

pub trait CacheArenaConfig {
    const ENTRIES: usize;
    const POOL_COUNT: usize;
    const POOL_SIZE: usize;
    const LEASES: usize;
}

pub struct CacheArena<TKey,TReference,TItem,TConfig> where
    TReference: Key,
{
    slotmap: SlotMap<TReference,SlotMapItem<TKey,TItem>>,
    pools: KeyedPools<TKey,TReference,TConfig>,
    phantom_config: std::marker::PhantomData<TConfig>
}

impl<TKey,TReference,TItem,TConfig> Default for CacheArena<TKey,TReference,TItem,TConfig> where
    TReference: Key,
    TKey: Eq + Copy + Hash,
    TConfig: CacheArenaConfig
{
    fn default() -> Self {
        return Self::new();
    }
}

impl<TKey,TReference,TItem,TConfig> CacheArena<TKey,TReference,TItem,TConfig> where
    TReference: Key
{
    pub fn get(&self,reference: TReference) -> Result<&TItem,CacheArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.get(reference) else {
            return Err(CacheArenaError::ExpiredReference(reference));
        };
        return Ok(&item.value);
    }
    pub fn get_mut(&mut self,reference: TReference) -> Result<&mut TItem,CacheArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.get_mut(reference) else {
            return Err(CacheArenaError::ExpiredReference(reference));
        };
        return Ok(&mut item.value);
    }
}

impl<TKey,TReference,TItem,TConfig> CacheArena<TKey,TReference,TItem,TConfig> where
    TReference: Key,
    TKey: Eq + Copy + Hash,
    TConfig: CacheArenaConfig 
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

    pub fn insert(&mut self,key: TKey,item: TItem) {
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

    pub fn remove(&mut self,reference: TReference) -> Result<TItem,CacheArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.remove(reference) else {
            return Err(CacheArenaError::ExpiredReference(reference));
        };
        let Some(KeyData{ key, index, pool_target }) = item.key_data else {
            return Ok(item.value);
        };

        let pool = match pool_target {
            PoolTarget::Cache => {
                let Some(cache) = self.pools.get_cache_mut(&key) else {
                    return Err(CacheArenaError::MissingKeyedPool(key));
                };
                cache
            },
            PoolTarget::Lease => self.pools.get_lease_pool_mut(),
        };

        pool.swap_remove(index);

        let Some(swapped_item_reference) = pool.get(index).cloned() else {
            return Err(CacheArenaError::PoolSwapAliasing(key,index));
        };
        let Some(swapped_item) = self.slotmap.get_mut(swapped_item_reference) else {
            return Err(CacheArenaError::ExpiredReference(swapped_item_reference)); 
        };
        let Some(swapped_item_key_data) = &mut swapped_item.key_data else {
            return Err(CacheArenaError::KeylessReference(swapped_item_reference));
        };
        swapped_item_key_data.index = index;

        return Ok(item.value);
    }

    fn pool_swap<PoolStrategy: PoolSelector<TKey,TReference>>(&mut self,reference: TReference) -> Result<(),CacheArenaError<TKey,TReference>> {
        let Some(item) = self.slotmap.get_mut(reference) else {
            return Err(CacheArenaError::ExpiredReference(reference)); 
        };
        let Some(key_data) = &mut item.key_data else {
            return Err(CacheArenaError::KeylessReference(reference));
        };

        let PoolOriginDestination { origin, destination, target } = PoolStrategy::order(match self.pools.get_cache_and_lease_mut(&key_data.key) {
            Some(value) => value,
            None => return Err(CacheArenaError::MissingKeyedPool(key_data.key)),
        });

        if key_data.pool_target == target {
            return match target {
                PoolTarget::Cache => Err(CacheArenaError::NotInLease(reference,key_data.key)),
                PoolTarget::Lease => Err(CacheArenaError::NotInCache(reference,key_data.key)),
            }
        }

        let origin_index = key_data.index;
        key_data.index = destination.len();
        key_data.pool_target = target;

        destination.push(reference);
        origin.swap_remove(key_data.index);

        let Some(swapped_item_reference) = origin.get(origin_index).cloned() else {
            return Err(CacheArenaError::PoolSwapAliasing(key_data.key,origin_index));
        };
        let Some(swapped_item) = self.slotmap.get_mut(swapped_item_reference) else {
            return Err(CacheArenaError::ExpiredReference(swapped_item_reference)); 
        };
        let Some(swapped_item_key_data) = &mut swapped_item.key_data else {
            return Err(CacheArenaError::KeylessReference(swapped_item_reference));
        };
        swapped_item_key_data.index = origin_index;
        
        return Ok(());
    }

    pub fn start_lease(&mut self,key: TKey) -> Result<TReference,CacheArenaError<TKey,TReference>> {
        let Some(cache) = self.pools.get_cache_mut(&key) else {
            return Err(CacheArenaError::MissingKeyedPool(key));
        };
        let Some(reference) = cache.last().cloned() else {
            return Err(CacheArenaError::EmptyKeyedPool(key));
        };
        self.pool_swap::<MoveToLease>(reference)?;
        return Ok(reference);
    }

    pub fn end_lease(&mut self,reference: TReference) -> Result<(),CacheArenaError<TKey,TReference>> {
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
