use anyhow::{Ok, Result};
use digsite::game::digsites::DigSite;
use rand::{prelude::*, rngs};

fn main() -> Result<()> {
    test()
}

fn test() -> Result<()> {
    let mut ds = DigSite::new(10, 10, 15, 5, 5);

    let mut rng = rngs::StdRng::from_entropy();
    ds.assign_bones(&mut rng)?;

    ds.print();

    Ok(())
}
