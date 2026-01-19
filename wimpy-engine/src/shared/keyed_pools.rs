use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::shared::cache_arena::CacheArenaConfig;

pub type Pool<T> = Vec<T>;

#[derive(PartialEq)]
pub enum PoolTarget {
    Cache,
    Lease,
}

pub struct MoveToLease;
pub struct MoveToCache;


pub struct KeyedPools<TKey,T,TConfig> {
    leases: Pool<T>,
    cache_container: HashMap<TKey,Pool<T>>,
    phantom_config: PhantomData<TConfig>
}

pub struct PoolPair<'a,T> {
    pub cache: &'a mut Pool<T>,
    pub lease: &'a mut Pool<T>,
}

pub struct PoolOriginDestination<'a,T> {
    pub origin: &'a mut Pool<T>,
    pub destination: &'a mut Pool<T>,
    pub target: PoolTarget,
}

impl<TKey,T,TConfig> KeyedPools<TKey,T,TConfig> where
    TKey: Eq + Hash,
    TConfig: CacheArenaConfig 
{
    fn create_pool() -> Pool<T> {
        return Vec::with_capacity(TConfig::POOL_SIZE);
    }

    pub fn new() -> Self {
        return Self {
            cache_container: HashMap::with_capacity(TConfig::POOL_COUNT),
            leases: Vec::with_capacity(TConfig::LEASES),
            phantom_config: PhantomData
        };
    }

    pub fn ensure_cache(&mut self,key: TKey) {
        self.cache_container.entry(key).or_insert_with(Self::create_pool);
    }

    pub fn get_or_create_cache_mut(&mut self,key: TKey) -> &mut Pool<T> {
        return self.cache_container.entry(key).or_insert_with(Self::create_pool);
    }

    pub fn get_cache_mut(&mut self,key: &TKey) -> Option<&mut Pool<T>> {
        return self.cache_container.get_mut(key);
    }

    pub fn get_cache_and_lease_mut<'a>(&'a mut self,key: &TKey) -> Option<PoolPair<'a,T>> {
        return match self.cache_container.get_mut(key) {
            Some(cache_pool) => Some(PoolPair {
                cache: cache_pool,
                lease: &mut self.leases
            }),
            None => None,
        };
    }

    pub fn pop_lease(&mut self) -> Option<T> {
        return self.leases.pop();
    }

    pub fn get_lease_pool_mut(&mut self) -> &mut Pool<T> {
        return &mut self.leases;
    }
}

pub trait PoolSelector<TKey,T> {
    fn order<'a>(pool_pair: PoolPair<'a,T>) -> PoolOriginDestination<'a,T>;
}

impl<TKey,T> PoolSelector<TKey,T> for MoveToCache {
    fn order<'a>(pool_pair: PoolPair<'a,T>) -> PoolOriginDestination<'a,T> {
        return PoolOriginDestination {
            origin: pool_pair.lease,
            destination: pool_pair.cache,
            target: PoolTarget::Cache
        };
    }
}

impl<TKey,T> PoolSelector<TKey,T> for MoveToLease {
    fn order<'a>(pool_pair: PoolPair<'a,T>) -> PoolOriginDestination<'a,T> {
        return PoolOriginDestination {
            origin: pool_pair.cache,
            destination: pool_pair.lease,
            target: PoolTarget::Lease
        };
    }
}
