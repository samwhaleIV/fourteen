use std::{collections::{HashMap,VecDeque}};
use std::hash::{DefaultHasher,Hasher};
use rand::{RngCore, SeedableRng, rngs::SmallRng};

const DEFAULT_GROWTH_FACTOR: u32 = 3;
const DEFAULT_RNG_SEED: u64 = 1;

pub struct NamedCache<T> {
    id_counter: usize,
    growth_factor: u32,
    returned_counter_ids: VecDeque<usize>,
    items: Vec<Option<NamedCacheItem<T>>>,
    item_search_table: HashMap<String,CacheItemReference>,
    rng: SmallRng
}

struct NamedCacheItem<T> {
    hash: u64,
    salt: u64,
    name: String,
    value: T,
}

impl<T> NamedCacheItem<T> {
    pub fn matches(&self,reference: &CacheItemReference) -> bool {
        return self.hash == reference.hash && self.salt == reference.salt;
    }
}

/* How to guarantee that a provided cache item reference "belongs" to the right cache? */
#[derive(Copy,Clone)]
pub struct CacheItemReference {
    id: usize,
    hash: u64,
    salt: u64,
}

impl<T> Default for NamedCache<T> {
    fn default() -> Self {
        let growth_factor = DEFAULT_GROWTH_FACTOR;
        let rng_seed = DEFAULT_RNG_SEED;

        let size = get_growth_factor_size(growth_factor);
        let mut items = Vec::with_capacity(size);
        items.resize_with(size,Default::default);

        Self {
            id_counter: 0,
            growth_factor,
            returned_counter_ids: Default::default(),
            items,
            item_search_table: Default::default(),
            rng: SmallRng::seed_from_u64(rng_seed)
        }
    }
}

fn hash_name(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(name.as_bytes());
    return hasher.finish();
}

impl<T> NamedCache<T> {

    pub fn store_item(&mut self,name: &str,item: T) -> CacheItemReference {
        if self.item_search_table.contains_key(name) {
            panic!("Named item '{}' already exists!",name);
        }

        let id = match self.returned_counter_ids.pop_front() {
            Some(id) => id,
            None => {
                let id = self.id_counter;
                self.id_counter += 1;
                id
            },
        };

        
        let hash = hash_name(name);
        let salt = self.rng.next_u64();

        let named_cached_item = NamedCacheItem {
            value: item,
            name: name.to_string(),
            salt,
            hash
        };

        if id >= self.items.len() {
            if self.items.len() == self.items.capacity() {
                self.growth_factor += 1;
            }
            let desired_size = get_growth_factor_size(self.growth_factor);
            self.items.resize_with(desired_size,Default::default);
        }

        self.items[id] = Some(named_cached_item);
        let reference = CacheItemReference { id, salt, hash };
        self.item_search_table.insert(name.to_string(),reference);

        return reference;
    }

    pub fn borrow_item(&self,reference: &CacheItemReference) -> &T {
        if reference.id >= self.items.len() {
            panic!("Bad cache item reference. ID '{}' out of bounds.",reference.id);
        } else if let Some(item) = &self.items[reference.id] {
            if !item.matches(reference) {
                panic!("Dangling pointer/aliasing occured due to expired item reference.");
            }
            return &item.value;
        } else {
            panic!("Bad cache item reference. No value at index '{}'.",reference.id);
        }
    }

    pub fn remove_item(&mut self,reference: &CacheItemReference) -> T {
        if reference.id >= self.items.len() {
            panic!("Bad cache item reference. ID '{}' out of bounds.",reference.id);
        } else if let Some(named_item) = &self.items[reference.id] {
            if !named_item.matches(reference) {
                panic!("Dangling pointer/aliasing occured due to expired item reference.");
            }
            self.item_search_table.remove(&named_item.name);
            self.returned_counter_ids.push_back(reference.id);

            /* Dynamic downsizing might be too complex to implement because of fragmentation of the vector indices, but not impossible to solve. */

            return self.items[reference.id].take().unwrap().value;
        } else {
            panic!("Bad cache item reference. No value at index '{}'.",reference.id);
        }
    }

    pub fn get_reference(&self,name: &str) -> Option<CacheItemReference> {
        if let Some(reference) = self.item_search_table.get(name) {
            return Some(reference.clone());
        } else {
            return None;
        }
    }

}

const fn get_growth_factor_size(growth_factor: u32) -> usize {
    return usize::pow(2,growth_factor);
}
