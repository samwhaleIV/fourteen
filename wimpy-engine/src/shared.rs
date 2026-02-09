use std::ops::Range;
use cgmath::Matrix4;

mod color;
mod layout;
mod cache_arena;
mod keyed_pools;
mod pool;

pub use color::*;
pub use layout::*;
pub use cache_arena::*;
pub use keyed_pools::*;
pub use pool::*;

pub const fn downcast_range(value: Range<usize>) -> Range<u32> {
    return Range {
        start: value.start as u32,
        end: value.end as u32,
    };
}

pub const fn get_identity_matrix() -> Matrix4<f32> {
    return Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}
pub trait IdentityTrait<T> {
    fn get(&self) -> &T;
    fn get_mut(&mut self) -> &mut T;
}
