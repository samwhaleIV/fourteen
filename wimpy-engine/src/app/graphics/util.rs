use std::ops::Range;

pub const fn downcast_range(value: Range<usize>) -> Range<u32> {
    return Range {
        start: value.start as u32,
        end: value.end as u32,
    };
}
