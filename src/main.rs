use serde::{Deserialize, Serialize};
use markdown::Block;

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

impl Style {

    // looks for a Styles.toml file in the current directory and 
    // deserialises the file into a vector of Style structs
    fn get_styles() -> Vec<Style> {
        // read an existing Styles.toml or create it and populate it with the default style
        match std::fs::read_to_string("Styles.toml") {
            Ok(s) => {
                return toml::from_str(&s)
                    .expect("Failed to parse Styles.toml - check the file is valid TOML.");
            },
            Err(_) => {
                let mut styles = Vec::new();
                styles.push(Style::default());
                return styles;
            }
        };
    }

}

struct Section {
    // A block of tokenized markdown content.
    block: Block,
    // The style alias that the corresponding Block should be formatted in.
    style: String,
    // The markdown file globs to which define this sections content.
    // Blocks from files that match these globs will be added to this section
    // and be formatted with the style alias defined by the style field.
    includes: Vec<String>,
}

fn main() {
    let styles = Style::get_styles();
    println!("{:?}", styles);
}