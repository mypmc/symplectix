use bits::Word;
use std::borrow::Cow;
use std::iter::successors;

#[test]
fn bits_is_implemented() {
    fn _test<T>()
    where
        T: ?Sized
            + bits::ops::BitLen
            + bits::ops::BitCount
            + bits::ops::BitAll
            + bits::ops::BitAny
            + bits::ops::BitRank
            + bits::ops::BitExcess
            + bits::ops::BitSelect
            + bits::ops::BitGet,
    {
    }

    _test::<&u8>();
    _test::<[u8; 1]>();
    _test::<&[u8; 1]>();
    _test::<[u8]>();
    _test::<&[u8]>();
    _test::<Vec<[u8; 1]>>();
    _test::<&Vec<[u8; 2]>>();
    _test::<Box<[u8; 3]>>();
    _test::<[Box<[u8; 3]>]>();
    _test::<&Box<[u8; 4]>>();
    _test::<Cow<[u8; 1000]>>();
    _test::<Cow<Box<[u8; 2000]>>>();
}

fn ones<T: Word>(word: T) -> impl Iterator<Item = usize> {
    successors(Some(word), |&n| {
        let m = n & !n.lsb();
        bits::any(&m).then(|| m)
    })
    .map(Word::count_t0)
}

#[test]
fn next_set_bit() {
    let n: u32 = 0b_0101_0101;
    let mut ones = ones(n);

    assert_eq!(ones.next(), Some(0));
    assert_eq!(ones.next(), Some(2));
    assert_eq!(ones.next(), Some(4));
    assert_eq!(ones.next(), Some(6));
    assert_eq!(ones.next(), None);
}

#[test]
fn ones_select1() {
    let n: u32 = 0b_0101_0101;
    let mut ones = ones(n);
    for c in 0..bits::count_1(&n) {
        assert_eq!(ones.next(), bits::select_1(&n, c));
    }
}

fn rank_for_empty_range<T>(bits: &T)
where
    T: ?Sized + bits::ops::BitRank,
{
    assert_eq!(bits::rank_0(bits, 0..0), 0);
    assert_eq!(bits::rank_0(bits, 1..1), 0);
    assert_eq!(bits::rank_0(bits, 2..2), 0);
    assert_eq!(bits::rank_0(bits, 7..7), 0);

    assert_eq!(bits::rank_1(bits, 0..0), 0);
    assert_eq!(bits::rank_1(bits, 1..1), 0);
    assert_eq!(bits::rank_1(bits, 2..2), 0);
    assert_eq!(bits::rank_1(bits, 7..7), 0);
}

fn rank_0_plus_rank_1<T>(bits: &T, r: core::ops::Range<usize>)
where
    T: ?Sized + bits::ops::BitRank,
{
    assert_eq!(
        bits::rank_0(bits, r.clone()) + bits::rank_1(bits, r.clone()),
        r.len(),
    );
}

#[test]
fn bit_rank() {
    rank_for_empty_range::<u8>(&!0);
    rank_for_empty_range::<[u8]>(&[!0, !0, !0, !0]);
    rank_for_empty_range::<[bool]>(&[true, true, true, true, true, true, true, true]);

    rank_0_plus_rank_1::<u64>(&0b_1010_1010, 0..10);
    rank_0_plus_rank_1::<u64>(&0b_1010_1010, 7..20);
    rank_0_plus_rank_1::<[u8]>(&[!0, 0b_1010_1010, !0, 0b_1010_1010], 0..10);
    rank_0_plus_rank_1::<[u8]>(&[!0, 0b_1010_1010, !0, 0b_1010_1010], 7..20);
}