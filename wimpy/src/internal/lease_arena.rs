use std::{collections::{HashMap, VecDeque}, hash::Hash};
use generational_arena::{Arena, Index};

pub struct LeaseArena<TKey,TValue> {
    all_items: Arena<TValue>,
    keyed_items: HashMap<TKey,VecDeque<Index>>,
    leased_items: HashMap<Index,TKey>
}

impl<TKey: Hash + Eq + Copy,TValue> Default for LeaseArena<TKey,TValue> {
    fn default() -> Self {
        Self {
            all_items: Default::default(),
            keyed_items: Default::default(),
            leased_items: Default::default()
        }
    }
}

#[allow(unused)]
impl<TKey: Hash + Eq + Copy,TValue> LeaseArena<TKey,TValue> {

    pub fn create_with_values(
        all_items: Arena<TValue>,
        keyed_items: HashMap<TKey,VecDeque<Index>>,
    ) -> Self {
        return Self {
            all_items,
            keyed_items,
            leased_items: Default::default()
        }
    }

    pub fn insert_leasable(&mut self,key: TKey,item: TValue) -> Index {
        let index = self.all_items.insert(item);

        let mut keyed_item_pool = {
            if let Some(value) = self.keyed_items.remove(&key) {
                value
            } else {
                VecDeque::<Index>::new()
            }
        };

        keyed_item_pool.push_back(index);
        self.keyed_items.insert(key,keyed_item_pool);

        return index;
    }

    /* For immediate consumption */
    pub fn insert_leasable_and_take(&mut self,key: TKey,item: TValue) -> Index {
        let index = self.all_items.insert(item);

        if !self.keyed_items.contains_key(&key) {
            self.keyed_items.insert(key,VecDeque::<Index>::new());
        }

        self.leased_items.insert(index,key);

        return index;
    }

    pub fn insert_keyless(&mut self,item: TValue) -> Index {
        return self.all_items.insert(item);
    }

    pub fn remove(&mut self,index: Index) {
        self.all_items.remove(index);
        /* Can't remove from the keyed or leased set because we can't know what key this index may have (if any) */
    }

    pub fn get(&self,reference: Index) -> &TValue {
        if let Some(item) = self.all_items.get(reference) {
            return item;
        } else {
            panic!("Item not found in arena with this index!");
        }
    }

    pub fn try_request_lease(&mut self,key: TKey) -> Option<Index> {
        let mut keyed_item_pool = {
            if let Some(value) = self.keyed_items.remove(&key) {
                value
            } else {
                VecDeque::<Index>::new()
            }
        };

        let index = {
            if let Some(index) = keyed_item_pool.pop_back() {
                self.leased_items.insert(index,key);
                index
            } else {
                self.keyed_items.insert(key,keyed_item_pool);
                return None;
            }
        };

        return Some(index);
    }

    pub fn end_lease(&mut self,lease: Index) {
        if let Some(key) = self.leased_items.remove(&lease) {
            if let Some(mut keyed_item_pool) = self.keyed_items.remove(&key) {     
                keyed_item_pool.push_back(lease);
                self.keyed_items.insert(key,keyed_item_pool);
            } else {
                panic!("Keyed item group not found.");
            }
        } else {
            panic!("Reference not found in lease cache!");
        }
    }

    pub fn end_all_leases(&mut self) {
        for (lease,key) in self.leased_items.iter() {
            if let Some(mut keyed_item_pool) = self.keyed_items.remove(&key) {     
                keyed_item_pool.push_back(*lease);
                self.keyed_items.insert(*key,keyed_item_pool);
            } else {
                panic!("Keyed item group not found.");
            }
        }
        self.leased_items.clear();
    }

    pub fn read_lease(&self,lease: Index) -> &TValue {
        if !self.leased_items.contains_key(&lease) {
            panic!("Reference not found in lease cache!");
        }

        if let Some(value) = self.all_items.get(lease) {
            return value;
        }

        panic!("Texture reference not found in cache!");
    }
}
