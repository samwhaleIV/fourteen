pub struct VecPool<T,const INNER_CAPACITY: usize> {
    values: Vec<Vec<T>>
}

impl<T,const INNER_CAPACITY: usize> Default for VecPool<T,INNER_CAPACITY> {
    fn default() -> Self {
        Self {
            values: Default::default()
        }
    }
}

impl<T,const INNER_CAPACITY: usize> VecPool<T,INNER_CAPACITY> {
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

    pub fn return_item(&mut self,mut item: Vec<T>) {
        item.clear();
        self.values.push(item);
    }
}

pub struct StringPool<const INNER_CAPACITY: usize> {
    values: Vec<String>
}

impl<const INNER_CAPACITY: usize> Default for StringPool<INNER_CAPACITY> {
    fn default() -> Self {
        Self {
            values: Default::default()
        }
    }
}

impl<const INNER_CAPACITY: usize> StringPool<INNER_CAPACITY> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    pub fn take_item(&mut self) -> String {
        match self.values.pop() {
            Some(value) => value,
            None => String::with_capacity(INNER_CAPACITY),
        }
    }

    pub fn return_item(&mut self,mut item: String) {
        item.clear();
        self.values.push(item);
    }
}
