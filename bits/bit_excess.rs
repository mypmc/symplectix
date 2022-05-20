use crate as bits;
use core::ops::RangeBounds;

pub trait BitExcess: bits::bit_rank::BitRank {
    fn excess_1<Index: RangeBounds<usize>>(&self, index: Index) -> usize;
    fn excess_0<Index: RangeBounds<usize>>(&self, index: Index) -> usize;
}

impl<T: bits::bit_rank::BitRank> BitExcess for T {
    #[inline]
    fn excess_1<Index: RangeBounds<usize>>(&self, index: Index) -> usize {
        let (i, j) = bits::to_range(&index, 0, bits::len(self));
        let rank1 = self.rank_1(i..j);
        let rank0 = self.rank_0(i..j);
        assert!(rank1 >= rank0);
        rank1 - rank0
    }

    #[inline]
    fn excess_0<Index: RangeBounds<usize>>(&self, index: Index) -> usize {
        let (i, j) = bits::to_range(&index, 0, bits::len(self));
        let rank1 = self.rank_1(i..j);
        let rank0 = self.rank_0(i..j);
        assert!(rank0 >= rank1);
        rank0 - rank1
    }
}
