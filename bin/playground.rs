use anyhow::{Ok, Result};
use digsite::{
    game::digsites::DigSite,
    geometry::{Point, Size},
};
use rand::{prelude::*, rngs};

fn main() -> Result<()> {
    test()
}

fn test() -> Result<()> {
    let mut rng = rngs::StdRng::from_entropy();

    let ds = DigSite::generate(&mut rng, Size { x: 10, y: 10 }, 10, Point { x: 5, y: 5 })?;

    ds.print();

    Ok(())
}
