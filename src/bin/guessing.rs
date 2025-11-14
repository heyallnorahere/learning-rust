use std::cmp;
use std::io;
use std::io::Write;
use std::num;

use rand::RngCore;

fn prompt_for_guess() -> Result<u32, num::ParseIntError> {
    print!("Guess a non-negative integer: ");
    io::stdout().flush().expect("Failed to flush stdout!");

    let mut value = String::new();
    io::stdin().read_line(&mut value).expect("Failed to read from stdin");

    return value.trim().parse();
}

fn main() {
    let max = 50;

    println!("I'm thinking of a number between 1 and {max}.");
    let secret = (rand::rng().next_u32() % max) + 1;

    loop {
        let guess = match prompt_for_guess() {
            Ok(input) => input,
            Err(e) => {
                println!("Must input a non-negative integer. ({e:?})");
                continue;
            },
        };

        match guess.cmp(&secret) {
            cmp::Ordering::Less => println!("Guess higher!"),
            cmp::Ordering::Greater => println!("Guess lower!"),
            cmp::Ordering::Equal => break,
        };
    }

    println!("You guessed it! The number was {secret}.");
}
