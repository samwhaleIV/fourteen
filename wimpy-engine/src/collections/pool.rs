pub struct VecPool<T,const DEFAULT_CAPACITY: usize> {
    values: Vec<Vec<T>>
}

impl<T,const INNER_CAPACITY: usize> VecPool<T,INNER_CAPACITY> {
    pub fn new() -> Self {
        Self {
            values: Default::default(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn take_item(&mut self) -> Vec<T> {
        match self.values.pop() {
            Some(value) => value,
            None => Vec::with_capacity(INNER_CAPACITY),
        }
    }

    pub fn return_item(&mut self,item: Vec<T>) {
        self.values.push(item);
    }
}
