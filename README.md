
# Tiny RNG, a minimal random number generator

Examples:

```rust
    use tiny_rng::{Rng,Rand};

    fn main() {
        let seed: u64 = 0;
        let mut rng = Rng::new(seed);
        
        // Throw a dice 10 times.
        let v: Vec<u32> = (0..10).map(|_| rng.rand_range_u32(1,6)).collect();
        println!("{:?}\n",v);

        // Shuffle the array [0,1,...,99].
        let mut v: Vec<u32> = (0..100).collect();
        rng.shuffle(&mut v);
        println!("{:?}\n",v);
    }
```

