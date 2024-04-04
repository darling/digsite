use digsite::game::digsites::DigSite;
use rand::{prelude::*, rngs};

fn main() {
    test();
}

fn test() {
    let mut ds = DigSite::new(10, 10, 15, 5, 5);

    let rng = rngs::StdRng::from_entropy();
    ds.assign_bombs(rng);

    ds.print()
}
