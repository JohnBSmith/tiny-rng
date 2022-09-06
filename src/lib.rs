
/*! ## Tiny RNG, a minimal random number generator
Warning: Not cryptographically secure.

Proven mathematical methods are applied to obtain unbiased samples.
Specifically, rejection sampling is applied to obtain samples from
the uniform distribution on an integer range and Fisher–Yates shuffle is
applied to obtain a random permutation from the uniform distribution
on the set of all permutations.

```
use tiny_rng::{Rng, Rand};

fn main() {
    let mut rng = Rng::from_seed(0);
    
    // Throw a dice:
    println!("{}", rng.rand_range_u32(1, 7));

    // Choose a random color:
    let colors = ["red", "green", "blue"];
    println!("{}", rng.choice(&colors));

    // Shuffle an array:
    let mut a = [1, 2, 3, 4];
    rng.shuffle(&mut a);
    println!("{:?}", a);
}
```
*/

#![cfg_attr(not(test), no_std)]

#[cfg(feature = "std")]
extern crate std;

fn wrapping_next_power_of_two_u32(x: u32) -> u32 {
    const H: u32 = 1 << 31;
    if x <= H {x.next_power_of_two()} else {0}
}

fn wrapping_next_power_of_two_u64(x: u64) -> u64 {
    const H: u64 = 1 << 63;
    if x <= H {x.next_power_of_two()} else {0}
}

/** Provided utilities.

This interface permits to hide the used engine:
```
fn rng_from_seed(seed: u64) -> impl Rand {
    Rng::from_seed(seed)
}
```
*/
pub trait Rand: Sized {
    /// To obtain reproducible results.
    /// If feature `std` is enabled, `from_time()` may be used instead,
    /// to use system time as microseconds as a seed. Apply
    /// `from_seed(random_seed())` to establish a faster local RNG
    /// from a global seed RNG.
    fn from_seed(seed: u64) -> Self;

    #[cfg(feature = "std")]
    /// Use system time as micro seconds as the seed.
    /// Needs feature `std` to be enabled.
    fn from_time() -> Self {
        use std::time::SystemTime;
        let time_seed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|e| e.duration()).as_micros() as u64;
        Self::from_seed(time_seed)
    }

    /// A sample from the uniform distribution on `0..=u32::MAX`.
    fn rand_u32(&mut self) -> u32;

    /// A sample from the uniform distribution on `0..=u8::MAX`.
    #[inline]
    fn rand_u8(&mut self) -> u8 {
        self.rand_u32() as u8
    }

    /// A sample from the uniform distribution on `0..=u16::MAX`.
    #[inline]
    fn rand_u16(&mut self) -> u16 {
        self.rand_u32() as u16
    }

    /// A sample from the uniform distribution on `0..=u64::MAX`.
    #[inline]
    fn rand_u64(&mut self) -> u64 {
        (self.rand_u32() as u64) << 32 | (self.rand_u32() as u64)
    }

    /// A sample from the uniform distribution on `0..=usize::MAX`.
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn rand_usize(&mut self) -> usize {
        self.rand_u32() as usize
    }
    
    /// A sample from the uniform distribution on `0..=usize::MAX`.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn rand_usize(&mut self) -> usize {
        self.rand_u64() as usize
    }

    /// A sample from the uniform distribution on `0..m`.
    // Applies the idea of rejection sampling.
    fn rand_bounded_u32(&mut self, m: u32) -> u32 {
        let mask = wrapping_next_power_of_two_u32(m).wrapping_sub(1);
        loop {
            let x = mask & self.rand_u32();
            if x < m {return x;}
        }
    }

    /// A sample from the uniform distribution on `0..m`.
    fn rand_bounded_u64(&mut self, m: u64) -> u64 {
        let mask = wrapping_next_power_of_two_u64(m).wrapping_sub(1);
        loop {
            let x = mask & self.rand_u64();
            if x < m {return x;}
        }
    }

    /// A sample from the uniform distribution on `0..m`.
    #[cfg(target_pointer_width = "32")]
    fn rand_bounded_usize(&mut self, m: usize) -> usize {
        self.rand_bounded_u32(m as u32) as usize
    }

    /// A sample from the uniform distribution on `0..m`.
    #[cfg(target_pointer_width = "64")]
    fn rand_bounded_usize(&mut self, m: usize) -> usize {
        self.rand_bounded_u64(m as u64) as usize
    }

    /// A sample from the uniform distribution on `a..b`.
    fn rand_range_u32(&mut self, a: u32, b: u32) -> u32 {
       a + self.rand_bounded_u32(b - a)
    }
    
    /// A sample from the uniform distribution on `a..b`.
    fn rand_range_u64(&mut self, a: u64, b: u64) -> u64 {
       a + self.rand_bounded_u64(b - a)
    }

    /// A sample from the uniform distribution on `a..b`.
    fn rand_range_i32(&mut self, a: i32, b: i32) -> i32 {
        a + self.rand_bounded_u32((b - a) as u32) as i32
    }

    /// A sample from the uniform distribution on `a..b`.
    fn rand_range_i64(&mut self, a: i64, b: i64) -> i64 {
        a + self.rand_bounded_u64((b - a) as u64) as i64
    }

    /// A sample from the uniform distribution on the interval [0, 1).
    fn rand_f32(&mut self) -> f32 {
        self.rand_u32() as f32 * 2.3283064E-10
    }

    /// A sample from the uniform distribution on the interval [0, 1).
    fn rand_f64(&mut self) -> f64 {
        self.rand_u32() as f64 * 2.3283064365386963E-10
    }
    
    /// A sample from the uniform distribution on the non-empty slice.
    fn choice<'a, T>(&mut self, a: &'a [T]) -> &'a T {
        &a[self.rand_bounded_usize(a.len())]
    }

    /// Shuffle an array randomly. The method is called Fisher–Yates shuffle and has linear time complexity.
    fn shuffle<T>(&mut self, a: &mut [T]) {
        if a.is_empty() {return;}
        let mut i = a.len() - 1;
        while i > 0 {
            let j = self.rand_bounded_usize(i + 1);
            a.swap(i, j);
            i -= 1;
        }
    }

    /// Fill a buffer with random bytes.
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

/** A helper function to turn random number generation into an iterator.

Example:
```
use tiny_rng::{Rng, Rand, rand_iter};

fn main() {
    let mut rng = Rng::from_seed(0);
    for x in rand_iter(&mut rng, Rand::rand_u32).take(10) {
        println!("0x{:08x}", x);
    }
}
```
*/
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
following implementation is as described in Vignas article [1], with
the assignment a=23, b=17, c=26 described there as favorable, and is
also listed that way in the Firefox source code [2] and in
Wikipedia [3][4]. The internal state may be arbitrary,
but not (0, 0).

Since the low-order 32 bits of a random value of Xorshift128+ are not
supposed to pass some statistical tests, it is better to extract
the high-order 32 bits for rand_u32.

[1] Sebastiano Vigna: "Further scramblings of Marsaglia's
xorshift generators".
Journal of Computational and Applied Mathematics (Mai 2017).
arXiv:1404.0390. doi:10.1016/j.cam.2016.11.006.
https://arxiv.org/abs/1404.0390

[2] mfbt/XorShift128PlusRNG.h, retrieved 8 Feb. 2020.
https://github.com/mozilla/gecko-dev/blob/master/mfbt/XorShift128PlusRNG.h
-- Gecko is the current engine of Firefox.

[3] Xorshift. English Wikipedia, retrieved 8 Feb. 2020.
https://en.wikipedia.org/wiki/Xorshift

[4] Xorshift. German Wikipedia, retrievend 8 Feb. 2020.
https://de.wikipedia.org/wiki/Xorshift.
*/

/// Recommended engine. Currently Xorshift128+.
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
    /** A helper function to turn random number generation into an iterator.

    Example:

    ```
    use tiny_rng::{Rng, Rand};

    fn main() {
        let mut rng = Rng::from_seed(0);
        for x in rng.iter(Rand::rand_u32).take(10) {
            println!("0x{:08x}", x);
        }
    }
    ```
    */
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
