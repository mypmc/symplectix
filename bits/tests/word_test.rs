use bits::Word;
use core::{arch::x86_64, fmt::Debug};

#[test]
fn lsb() {
    let tests = [
        (0b0000_0000_u8, 0b0000_0000),
        (0b0000_0001_u8, 0b0000_0001),
        (0b0000_1100_u8, 0b0000_0100),
        (0b1001_0100_u8, 0b0000_0100),
        (0b1001_0000_u8, 0b0001_0000),
    ];

    for (n, want) in tests {
        assert_eq!(n.lsb(), want);
    }
}

#[test]
fn msb() {
    let tests = [
        (0b0000_0000_u8, 0b0000_0000),
        (0b0000_0001_u8, 0b0000_0001),
        (0b0000_1100_u8, 0b0000_1000),
        (0b1001_0100_u8, 0b1000_0000),
        (0b1001_0000_u8, 0b1000_0000),
    ];

    for (n, want) in tests {
        assert_eq!(n.msb(), want);
    }
}

trait Pdep {
    fn pdep(self, mask: Self) -> Self;
}

impl Pdep for u32 {
    fn pdep(self, mask: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("bmi2") {
                return unsafe { x86_64::_pdep_u32(self, mask) };
            }
        }
        _pdep(self, mask)
    }
}

impl Pdep for u64 {
    fn pdep(self, mask: Self) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("bmi2") {
                return unsafe { x86_64::_pdep_u64(self, mask) };
            }
        }
        _pdep(self, mask)
    }
}

fn _pdep<T: Word>(data: T, mut mask: T) -> T {
    let mut dest = T::NULL;
    for i in 0..T::BITS {
        if !mask.bit_any() {
            break;
        }
        if data.bit_get(i).unwrap() {
            dest |= mask.lsb();
        }
        mask &= mask - T::_1;
    }
    dest
}

fn pdep_test<T>(s: T, m: T, o: T)
where
    T: Word + Pdep + Debug,
{
    assert_eq!(s.pdep(m), o);
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    if is_x86_feature_detected!("bmi2") {
        assert_eq!(s.pdep(m), _pdep(s, m));
    }
}

macro_rules! pdep_test {
    ($T:ty) => {
        pdep_test::<$T>(
            0b_1011111010010011,
            0b_0110001110000101,
            0b_0000001000000101,
        );

        pdep_test::<$T>(
            0b_0000000000000000,
            0b_0110001110000100,
            0b_0000000000000000,
        );

        pdep_test::<$T>(
            0b_0000000000001000,
            0b_0110001110000100,
            0b_0000001000000000,
        );

        pdep_test::<$T>(
            0b_0000000000010000,
            0b_0110001110000100,
            0b_0010000000000000,
        );

        pdep_test::<$T>(
            0b_0000000000100000,
            0b_0110001110000100,
            0b_0100000000000000,
        );
    };
}

#[test]
fn pdep_u32() {
    pdep_test!(u32);
}

#[test]
fn pdep_u64() {
    pdep_test!(u64);
}
