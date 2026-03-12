use std::{collections::HashMap, hash::Hash};

struct SlotEntry<TKey> {
    owner: TKey,
    referenced: bool
}

struct ClockCacheSlots<TKey> {
    /// Slab of slot entries, 
    buffer: Vec<SlotEntry<TKey>>,
    /// Clock hand that points to `slots`
    hand: usize,
}

pub struct ClockCache<TKey> {
    slots: ClockCacheSlots<TKey>,
    /// Points external keys to the internal slab, `slots`
    map: HashMap<TKey,usize>,
}

pub struct SlotData<TKey> {
    pub key: TKey,
    pub slot: usize
}
pub struct CacheState<TKey> {
    pub slot: usize,
    pub feedback: Option<CacheInsertionFeedback<TKey>>
}

pub struct CacheInsertionFeedback<TKey> {
    /// The previous key that was in use at this slot
    /// 
    /// `None` if this is the first key registered to this slot
    pub old_key: Option<TKey>,
    /// The key that is assigned to this slot
    pub new_key: TKey,
}

impl<TKey> ClockCacheSlots<TKey>
where
    TKey: Copy
{
    fn get_evictee(&mut self) -> SlotData<TKey> {
        let len = self.buffer.len();
        assert!(len > 0);
        loop {
            let prev_hand = self.hand;
            self.hand = (prev_hand + 1) % len;

            let evictee = &mut self.buffer[prev_hand];

            if !evictee.referenced {
                return SlotData {
                    key: evictee.owner,
                    slot: prev_hand
                }
            }

            evictee.referenced = false;
        }
    }
}

impl<TKey> ClockCache<TKey>
where
    TKey: Hash + Eq + Copy
{
    pub fn new(capacity: usize) -> Self {
        Self {
            slots: ClockCacheSlots {
                buffer: Vec::with_capacity(capacity),
                hand: 0
            },
            map: HashMap::with_capacity(capacity),
        }
    }


    pub fn insert(&mut self,key: TKey) -> CacheState<TKey> {
        use std::collections::hash_map::Entry::*;
        match self.map.entry(key) {

            // Key already in cache
            Occupied(entry) => {
                let slot_id = *entry.get();
                if let Some(slot) = self.slots.buffer.get_mut(slot_id) {
                    slot.referenced = true;
                }
                CacheState {
                    slot: slot_id,
                    feedback: None,
                }
            },

            // Key not in cache
            Vacant(entry) => {
                let slot = self.slots.buffer.len();
                if slot < self.slots.buffer.capacity() {
                    self.slots.buffer.push(SlotEntry {
                        owner: key,
                        referenced: true,
                    });
                    entry.insert(slot);
                    CacheState {
                        slot,
                        feedback: Some(CacheInsertionFeedback {
                            old_key: None,
                            new_key: key,
                        })
                    }
                } else {
                    let evictee = self.slots.get_evictee();
                    entry.insert(evictee.slot);
                    self.map.remove(&evictee.key);
                    self.slots.buffer[evictee.slot] = SlotEntry {
                        owner: key,
                        referenced: true,
                    };
                    CacheState {
                        slot: evictee.slot,
                        feedback: Some(CacheInsertionFeedback {
                            old_key: Some(evictee.key),
                            new_key: key,
                        })
                    }
                }
            },
        }
    }

    pub fn get_slot_for_key(&self,key: TKey) -> Option<usize> {
        self.map.get(&key).copied()
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.slots.buffer.clear();
        self.slots.hand = 0;
    }
}
