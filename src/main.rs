use serde::{Deserialize, Serialize};

// A text formatting style definition.

#[derive(Serialize, Deserialize, Debug)]
struct Style {
    // the name that the style is refrenced by in a section configuration
    alias: String,
    // the font family for the style
    font: String,
    // the font size for the style
    size: u32,
    // the font color for the style
    color: String,
}

// implement default values for the Style struct
impl Default for Style {
    fn default() -> Self {
        Style {
            alias: String::from("default"),
            font: String::from("sans-serif"),
            size: 12,
            color: String::from("#000000"),
        }
    }
}

fn main() {
    println!("Hello, world!");
}