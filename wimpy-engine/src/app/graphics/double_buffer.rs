use super::prelude::*;

pub struct DoubleBuffer<T> {
    output_buffer: Buffer,
    input_buffer: Vec<T>,
}

impl<TItem> DoubleBuffer<TItem> {
    pub fn new(output_buffer: Buffer) -> Self {
        return Self {
            output_buffer,
            input_buffer: Vec::with_capacity(DEFAULT_DOUBLE_BUFFER_SIZE),
        }
    }
    pub fn get_output_buffer(&self) -> &Buffer {
        return &self.output_buffer;
    }

    pub fn push(&mut self,value: TItem) -> Range<usize> {
        let start = self.input_buffer.len();
        self.input_buffer.push(value);
        let end = self.input_buffer.len();
        return Range { start, end };
    }

    pub fn push_set<I>(&mut self,values: I) -> Range<usize>
    where
        I: IntoIterator<Item = TItem>
    {
        let start = self.input_buffer.len();
        self.input_buffer.extend(values);
        let end = self.input_buffer.len();
        return Range { start, end };
    }

    pub fn reset(&mut self) {
       self.input_buffer.clear();
    }
}

impl<TItem> DoubleBuffer<TItem>
where
    TItem: Pod + Zeroable
{
    pub fn write_out(&self,queue: &Queue) {
        if
            let Some(size) = NonZero::new((self.input_buffer.len() * size_of::<TItem>()) as u64) &&
            let Some(mut buffer_view) = queue.write_buffer_with(&self.output_buffer,0,size)
        {
            buffer_view.copy_from_slice(bytemuck::cast_slice(&self.input_buffer));
        }
    }
    pub fn write_out_with_padding(&self,queue: &Queue,padding: usize) {
        for (i,item) in self.input_buffer.iter().enumerate() {
            queue.write_buffer(&self.output_buffer,(i * padding) as u64,bytemuck::bytes_of(item));
        }
    }
}
