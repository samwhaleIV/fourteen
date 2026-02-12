use super::prelude::*;
use crate::collections::SetBuffers;
use slotmap::Key;

pub type DrawDataSetBuffers<T,TKey> = SetBuffers<T,TKey,DEFAULT_DRAW_DATA_SET_BUFFER_SIZE>;

pub trait SetBuffersSelector<T>: Key {
    fn select(pipelines: &mut RenderPipelines) -> &mut DrawDataSetBuffers<T,Self>;
}

impl GraphicsContext {
    pub fn create_draw_data_set_lease<T,TKey: SetBuffersSelector<T>>(&mut self,draw_data: &[T]) -> TKey {
        SetBuffersSelector::select(&mut self.pipelines).open_lease_with_entries(draw_data)
    }
    pub fn end_draw_data_set_lease<T,TKey: SetBuffersSelector<T>>(&mut self,key: TKey) -> Option<()> {
        SetBuffersSelector::select(&mut self.pipelines).end_lease(key)
    }
    pub fn insert_draw_data_set<T,TKey: SetBuffersSelector<T>>(&mut self,set: Vec<T>) -> TKey {
        SetBuffersSelector::select(&mut self.pipelines).insert_active(set)
    }
    pub fn remove_draw_data_set<T,TKey: SetBuffersSelector<T>>(&mut self,key: TKey) -> Option<Vec<T>> {
        SetBuffersSelector::select(&mut self.pipelines).remove_active(key)
    }
}
