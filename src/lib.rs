
//! ## Tiny RNG, a minimal random number generator
//! Warning: Not cryptographically secure.

#![cfg_attr(not(test), no_std)]

pub trait Rand {
    fn from_seed(seed: u64) -> Self;
    fn rand_u32(&mut self) -> u32;

    #[inline]
    fn rand_u8(&mut self) -> u8 {
        self.rand_u32() as u8
    }

    #[inline]
    fn rand_u16(&mut self) -> u16 {
        self.rand_u32() as u16
    }

    #[inline]
    fn rand_u64(&mut self) -> u64 {
        (self.rand_u32() as u64) << 32 | (self.rand_u32() as u64)
    }

    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn rand_usize(&mut self) -> usize {
        self.rand_u32() as usize
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn rand_usize(&mut self) -> usize {
        self.rand_u64() as usize
    }

    // Algorithm to generate an equidistributed random number in 0..m.
    // Applies the idea of rejection sampling.
    fn rand_bounded_u32(&mut self, m: u32) -> u32 {
        let threshold = m.wrapping_neg().wrapping_rem(m);
        loop {
            let r = self.rand_u32();
            if r >= threshold {
                return r.wrapping_rem(m);
            }
        }
    }

    fn rand_bounded_u64(&mut self, m: u64) -> u64 {
        let threshold = m.wrapping_neg().wrapping_rem(m);
        loop {
            let r = self.rand_u64();
            if r >= threshold {
                return r.wrapping_rem(m);
            }
        }
    }

    #[cfg(target_pointer_width = "32")]
    fn rand_bounded_usize(&mut self, m: usize) -> usize {
        self.rand_bounded_u32(m as u32) as usize
    }

    #[cfg(target_pointer_width = "64")]
    fn rand_bounded_usize(&mut self, m: usize) -> usize {
        self.rand_bounded_u64(m as u64) as usize
    }

    fn rand_range_u32(&mut self, a: u32, b: u32) -> u32 {
       a + self.rand_bounded_u32(b-a)
    }
    fn rand_range_u64(&mut self, a: u64, b: u64) -> u64 {
       a + self.rand_bounded_u64(b-a)
    }
    fn rand_range_i32(&mut self, a: i32, b: i32) -> i32 {
        a + self.rand_bounded_u32((b-a) as u32) as i32
    }
    fn rand_range_i64(&mut self, a: i64, b: i64) -> i64 {
        a + self.rand_bounded_u64((b-a) as u64) as i64
    }

    fn rand_f32(&mut self) -> f32 {
        self.rand_u32() as f32 * 2.3283064E-10
    }

    fn rand_f64(&mut self) -> f64 {
        self.rand_u32() as f64 * 2.3283064365386963E-10
    }

    fn shuffle<T>(&mut self, a: &mut [T]) {
        if a.is_empty() {return;}
        let mut i = a.len() - 1;
        while i > 0 {
            let j = self.rand_usize()%(i + 1);
            a.swap(i, j);
            i -= 1;
        }
    }

    fn fill(&mut self, a: &mut[u8]) {
        let mut x = self.rand_u32();
        let mut count = 3;
        for p in a {
            *p = x as u8;
            if count == 0 {
                x = self.rand_u32();
                count = 3;
            } else {
                x  >>= 8;
                count -= 1;
            }
        }
    }
}

pub fn rand_iter<'a, T: 'static, Generator: Rand>(
    rng: &'a mut Generator,
    rand: fn(&mut Generator) -> T
) -> impl 'a + Iterator<Item = T>
{
    core::iter::from_fn(move || Some(rand(rng)))
}

/*
Xorshift128+ is a modern algorithm for the fast generation of random
numbers of relatively high quality, which passes the most important
statistical tests. The generator is said to have a period length of
2^128-1 and to pass "BigCrush" from the test suite "TestU01". It should
be noted that this is by no means a cryptographic generator. The
following implementation is as described in [Vigna], with the
assignment a=23, b=17, c=26 described there as favorable, and is
also listed that way in the Firefox source code [1] and in
Wikipedia [2][3]. The internal state state may be arbitrary,
but not (0, 0).

Since the low-order 32 bits of a random value of Xorshift128+ are not
supposed to pass some statistical tests, it is better to extract
the high-order 32 bits for rand_u32.

[Vigna] Sebastiano Vigna: "Further scramblings of Marsaglia's
xorshift generators".
Journal of Computational and Applied Mathematics (Mai 2017).
arXiv:1404.0390. doi:10.1016/j.cam.2016.11.006.
https://arxiv.org/abs/1404.0390

[1] mfbt/XorShift128PlusRNG.h, retrieved 8 Feb. 2020.
https://github.com/mozilla/gecko-dev/blob/master/mfbt/XorShift128PlusRNG.h
-- Gecko is the current engine of Firefox.

[2] Xorshift. English Wikipedia, retrieved 8 Feb. 2020.
https://en.wikipedia.org/wiki/Xorshift

[3] Xorshift. German Wikipedia, retrievend 8 Feb. 2020.
https://de.wikipedia.org/wiki/Xorshift.
*/

pub struct Rng {
    state: (u64, u64)
}

impl Rand for Rng {
    fn from_seed(seed: u64) -> Self {
        Self {state: (
            seed ^ 0xf4dbdf2183dcefb7, // [crc32(b"0"), crc32(b"1")]
            seed ^ 0x1ad5be0d6dd28e9b  // [crc32(b"2"), crc32(b"3")]
        )}
    }

    fn rand_u64(&mut self) -> u64 {
        let (mut x, y) = self.state;
        self.state.0 = y;
        x ^= x << 23;
        self.state.1 = x ^ y ^ (x >> 17) ^ (y >> 26);
        self.state.1.wrapping_add(y)
    }

    #[inline]
    fn rand_u32(&mut self) -> u32 {
        (self.rand_u64() >> 32) as u32
    }
}

impl Rng {
    pub fn iter<'a, T: 'static>(&'a mut self, rand: fn(&mut Self) -> T)
    -> impl 'a + Iterator<Item = T>
    {
        rand_iter(self, rand)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Rng, Rand};

    fn rng_test<Generator: Rand>() {
        let mut rng = Generator::from_seed(0);
        println!("{:?}", rng.rand_range_u32(1, 6));
    }

    #[test]
    fn main() {
        rng_test::<Rng>();
    }
}
