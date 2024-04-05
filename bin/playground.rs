use anyhow::{Ok, Result};
use digsite::game::digsites::DigSite;
use rand::{prelude::*, rngs};

fn main() -> Result<()> {
    test()
}

fn test() -> Result<()> {
    for _ in 0..1 {
        let mut ds = DigSite::new(10, 10, 2, 5, 5);

        let mut rng = rngs::StdRng::from_entropy();
        ds.assign_bones(&mut rng)?;
    }

    Ok(())
}
