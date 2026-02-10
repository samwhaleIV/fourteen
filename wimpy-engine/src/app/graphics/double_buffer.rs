use super::prelude::*;

pub struct DoubleBuffer<T> {
    output_buffer: Buffer,
    input_buffer: Vec<T>,
}

const START_CAPACITY: usize = 64;

impl<TItem> DoubleBuffer<TItem> {
    //TODO: Implement starting capacity
    pub fn new(output_buffer: Buffer) -> Self {
        return Self {
            output_buffer,
            input_buffer: Vec::with_capacity(START_CAPACITY),
        }
    }
    pub fn get_output_buffer(&self) -> &Buffer {
        return &self.output_buffer;
    }
    pub fn push(&mut self,value: TItem) -> Range<usize> {
        let start = self.input_buffer.len();
        self.input_buffer.push(value);
        let end = start.saturating_add(1);
        return Range { start, end };
    }

    pub fn push_convert<T>(&mut self,value: &T) -> Range<usize>
    where
        for<'a> TItem: From<&'a T> 
    {
        let start = self.input_buffer.len();
        self.input_buffer.push(TItem::from(value));
        let end = start.saturating_add(1);
        return Range { start, end };
    }

    pub fn push_convert_all<T>(&mut self,values: &[T]) -> Range<usize>
    where
        for<'a> TItem: From<&'a T> 
    {
        let start = self.input_buffer.len();
        for value in values {
            self.input_buffer.push(TItem::from(value));
        }
        let end = start.saturating_add(values.len());
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

impl DoubleBuffer<QuadInstance> {
    pub fn write_quad(&mut self,render_pass: &mut RenderPass,draw_data: &DrawData2D) {
        let range = self.push_convert(draw_data.into());
        render_pass.draw_indexed(0..Pipeline2D::INDEX_BUFFER_SIZE,0,crate::shared::downcast_range(range));
    }
    pub fn write_quad_set(&mut self,render_pass: &mut RenderPass,draw_data: &[DrawData2D]) {
        let range = self.push_convert_all(draw_data);
        render_pass.draw_indexed(0..Pipeline2D::INDEX_BUFFER_SIZE,0,crate::shared::downcast_range(range));
    }
}
