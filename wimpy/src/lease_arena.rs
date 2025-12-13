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

    pub fn insert_leasable(&mut self,key: TKey,item: TValue,allow_lease: bool) -> Index {
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

    pub fn insert(&mut self,key: TKey,item: TValue) -> Index {
        return self.all_items.insert(item);
    }

    pub fn get(&self,reference: Index) -> &TValue {
        if let Some(item) = self.all_items.get(reference) {
            return item;
        } else {
            panic!("Item not found in arena with this index!");
        }
    }

    pub fn start_lease<F>(&mut self,key: TKey,generator: F) -> Index where F: Fn() -> TValue {
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
                let item = generator();
                let index = self.all_items.insert(item);
                index
            }
        };

        self.keyed_items.insert(key,keyed_item_pool);
        self.leased_items.insert(index,key);

        return index;
    }

    pub fn end_lease(&mut self,lease: Index) {
        if let Some(key) = self.leased_items.remove(&lease) {
            if let Some(mut keyed_item_pool) = self.keyed_items.remove(&key) {     
                keyed_item_pool.push_back(lease);
                self.keyed_items.insert(key,keyed_item_pool);
            } else {
                panic!("Keyed item group not found (keyed_items.value). By the way, this really should NOT happen. What have you done?");
            }
        } else {
            panic!("Reference not found in lease cache!");
        }
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
