use crate as bits;

pub trait BitAny: bits::bit_count::BitCount {
    #[inline]
    fn any(&self) -> bool {
        !bits::is_empty(self) && self.count_1() > 0
    }
}
