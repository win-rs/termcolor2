use std::str::FromStr;

use termcolor2::{self, Color};

fn main() {
    let color = Color::from_str("#89b4fa");
    println!("{:?}", color);
}
