use std::array;

pub struct MoveToFrontStack<T,const UNIQUE_VALUE_COUNT: usize> {
    stack: [T;UNIQUE_VALUE_COUNT],
    length: usize
}

impl<T,const SIZE: usize> Default for MoveToFrontStack<T,SIZE>
where
    T: Default + Copy
{
    fn default() -> Self {
        Self {
            stack: array::repeat(Default::default()),
            length: 0,
        }
    }
}

impl<T,const UNIQUE_VALUE_COUNT: usize> MoveToFrontStack<T,UNIQUE_VALUE_COUNT>
where
    T: Default + Copy + PartialEq,
{
    fn get_top_index(&self) -> usize {
        self.length.saturating_sub(1)
    }

    fn index_of(&self,value: T) -> Option<usize> {
        for i in 0..self.length {
            let Some(stack_value) = self.stack.get(i) else {
                break;
            };
            if value == *stack_value {
                return Some(i);
            }
        }
        return None;
    }

    fn remove_at(&mut self,index: usize) {
        for i in index..self.get_top_index() {
            let shift_value = self.stack.get(i + 1);
            self.stack[i] = match shift_value {
                Some(v) => *v,
                None => Default::default(),
            };
        }
    }

    pub fn peek(&self) -> T {
        match self.stack.get(self.get_top_index()) {
            Some(value) => *value,
            None => Default::default(),
        }
    }

    pub fn push(&mut self,value: T) {
        if let Some(removal_index) = self.index_of(value) {
            self.remove_at(removal_index);
        }
        self.length = (self.length + 1).min(UNIQUE_VALUE_COUNT);
        self.stack[self.get_top_index()] = value;
    }

    pub fn remove(&mut self,value: T) {
        let Some(removal_index) = self.index_of(value) else {
            return;
        };
        self.remove_at(removal_index);
        self.length = self.get_top_index();
        self.stack[self.length] = Default::default();
    }
}
