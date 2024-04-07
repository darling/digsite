use std::{env, io, process::Command};

use anyhow::{Ok, Result};
use digsite::{
    game::digsites::DigSite,
    geometry::{point::EMPTY_POINT, Point, Size},
};
use rand::{prelude::*, rngs};

fn main() -> Result<()> {
    test()
}

fn clear_terminal() {
    // Check the operating system
    let os_type = env::consts::OS;
    let clear_command = if os_type == "windows" { "cls" } else { "clear" };

    // Execute the appropriate command for the OS
    Command::new(clear_command)
        .status()
        .expect("Failed to clear terminal");
}

fn test() -> Result<()> {
    let mut rng = rngs::StdRng::from_entropy();

    let mut ds = DigSite::generate(
        &mut rng,
        Size { x: 10, y: 10 },
        15,
        Point { x: 4, y: 4 },
        Some(vec!['C']),
    )?;

    let mut input = String::new();

    loop {
        clear_terminal();
        ds.print();

        match io::stdin().read_line(&mut input) {
            Result::Ok(_) => ds.move_player(
                'C',
                match input.trim().to_lowercase().as_str() {
                    "w" => Point { x: 0, y: -1 },
                    "s" => Point { x: 0, y: 1 },
                    "a" => Point { x: -1, y: 0 },
                    "d" => Point { x: 1, y: 0 },
                    _ => EMPTY_POINT,
                },
            ),
            _ => {}
        }
    }

    Ok(())
}
