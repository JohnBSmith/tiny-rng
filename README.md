
## Tiny RNG, a minimal random number generator

Warning: Not cryptographically secure.

Examples:

```rust
use tiny_rng::{Rng, Rand};

fn main() {
    let mut rng = Rng::from_seed(0);
  
    println!("A u32 random number: 0x{:08x}", rng.rand_u32());
    println!("Throw a dice: {}", rng.rand_range_u32(1, 6));

    let a: Vec<u32> = rng.iter(Rand::rand_u32).take(4).collect();
    println!("An array of u32 random numbers: {:08x?}", a);
    
    let a: Vec<u32> = rng.iter(|rng| rng.rand_range_u32(1, 6))
        .take(4).collect();
    println!("An array of dice samples: {:?}", a);

    let mut a: Vec<u32> = (0..10).collect();
    rng.shuffle(&mut a);
    println!("A shuffled array: {:?}", a);
    
    let mut a: [u8;4] = [0, 0, 0, 0];
    rng.fill(&mut a);
    println!("Random bytes: {:?}", a);
}
```

