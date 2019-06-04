
//! ## Tiny RNG, a minimal random number generator
//! Warning: Not cryptographically secure.
//! * Use `Rng` for statistical purposes.
//! * Do not use `LcRng` for scientific purposes.
//! * Use `LcRng` just in case `Rng` is not fast enough.

#![cfg_attr(not(test), no_std)]

pub trait Rand {
    fn new(seed: u64) -> Self;
    fn rand_u32(&mut self) -> u32;

    fn rand_u8(&mut self) -> u8 {
        return self.rand_u32() as u8;
    }
    fn rand_u16(&mut self) -> u16 {
        return self.rand_u32() as u16;
    }
    fn rand_u64(&mut self) -> u64 {
        return (self.rand_u32() as u64)<<32 | (self.rand_u32() as u64);
    }

    #[cfg(target_pointer_width = "32")]
    fn rand_usize(&mut self) -> usize {
        return self.rand_u32() as usize;
    }

    #[cfg(target_pointer_width = "64")]
    fn rand_usize(&mut self) -> usize {
        return self.rand_u64() as usize;
    }
    
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
    fn rand_bounded_usize(&mut self, m: usize) -> usize{
        return self.rand_bounded_u32(m as u32) as usize;
    }

    #[cfg(target_pointer_width = "64")]
    fn rand_bounded_usize(&mut self, m: usize) -> usize{
        return self.rand_bounded_u64(m as u64) as usize;
    }

    fn rand_range_u32(&mut self, a: u32, b: u32) -> u32 {
       return a+self.rand_bounded_u32(b-a);
    }
    fn rand_range_u64(&mut self, a: u64, b: u64) -> u64 {
       return a+self.rand_bounded_u64(b-a);
    }
    fn rand_range_i32(&mut self, a: i32, b: i32) -> i32 {
        return a+self.rand_bounded_u32((b-a) as u32) as i32;
    }
    fn rand_range_i64(&mut self, a: i64, b: i64) -> i64 {
        return a+self.rand_bounded_u64((b-a) as u64) as i64;
    }

    fn rand_f32(&mut self) -> f32 {
        return self.rand_u32() as f32 * 2.3283064365386963E-10;
    }
    fn rand_f64(&mut self) -> f64 {
        return self.rand_u32() as f64 * 2.3283064365386963E-10;
    }

    fn shuffle<T>(&mut self, a: &mut [T]) {
        if a.len()==0 {return;}
        let mut i = a.len()-1;
        while i>0 {
            let j = (self.rand_u32() as usize)%(i+1);
            a.swap(i,j);
            i-=1;
        }
    }
}

// PCG-XSH-RR
pub struct Rng {
    state: u64
}
impl Rng {
    fn rand(&mut self) -> u32 {
        const MULTIPLIER: u64 = 6364136223846793005;
        const INC: u64 = 1442695040888963407;
        let oldstate = self.state;
        self.state = oldstate.wrapping_mul(MULTIPLIER).wrapping_add(INC|1);
        let xorshifted: u32 = ((
            (oldstate.wrapping_shr(18)) ^ oldstate
        ).wrapping_shr(27)) as u32;
        let rot: u32 = (oldstate.wrapping_shr(59)) as u32;
        return (xorshifted.wrapping_shr(rot)) | (
            xorshifted.wrapping_shl(rot.wrapping_neg() & 31)
        );
    }
}
impl Rand for Rng {
    fn new(seed: u64) -> Self {
        let mut rng = Self{state: 0};
        rng.rand();
        rng.state = rng.state.wrapping_add(seed);
        rng.rand();
        return rng;
    }
    fn rand_u32(&mut self) -> u32 {
        return self.rand();
    }
}

// LCG from "Numerical Recipes"
pub struct LcRng {
    state: u32
}
impl LcRng {
    fn rand(&mut self) -> u32 {
        const A: u32 = 1664525;
        const C: u32 = 1013904223;
        let y = self.state.wrapping_mul(A).wrapping_add(C);
        self.state = y;
        return y;
    }
}
impl Rand for LcRng {
    fn new(seed: u64) -> Self {
        return Self{state: seed as u32};
    }
    fn rand_u32(&mut self) -> u32 {
        return self.rand();
    }        
    fn rand_bounded_u32(&mut self, m: u32) -> u32 {
        let p = (self.rand_u32() as u64).wrapping_mul(m as u64);
        return p.wrapping_shr(32) as u32;
    }
    fn rand_bounded_u64(&mut self, m: u64) -> u64 {
        return self.rand_u64().wrapping_rem(m);
    }
}


#[cfg(test)]
mod tests {
    use crate::{Rng,LcRng,Rand};

    fn rng_test<Generator: Rand>() {
        let mut rng = Generator::new(0);
        let v: Vec<u32> = (0..10).map(|_| rng.rand_range_u32(1,6)).collect();
        println!("{:?}\n",v);

        let mut v: Vec<u32> = (0..100).collect();
        rng.shuffle(&mut v);
        println!("{:?}\n",v);
    }

    #[test]
    fn main() {
        rng_test::<Rng>();
        rng_test::<LcRng>();
    }
}
