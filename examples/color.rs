use std::str::FromStr;

use termcolor2::{self, Color};

fn main() {
    let color = Color::from_str("#89b4fa");
    let color_rgb = Color::from_str("rgb(75% 180 250)");

    println!("{:?}", color);
    println!("{:?}", color_rgb);
}
