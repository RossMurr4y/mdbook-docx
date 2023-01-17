extern crate docx_rs;
extern crate serde;
extern crate derive_more;
extern crate mdbook;

// newtype struct for markdown::ListItem that allows deserialization
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize)]
#[serde(try_from = "markdown::ListItem", into = "ListItem")]
pub(crate) struct ListItem(markdown::ListItem);
impl From<ListItem> for markdown::ListItem {
    fn from(item: ListItem) -> Self {
        item.0
    }
}
use std::convert::TryFrom;
impl TryFrom<markdown::ListItem> for ListItem {
    type Error = ();
    fn try_from(value: markdown::ListItem) -> Result<Self, Self::Error> {
        Ok(ListItem(value))
    }
}

// newtype struct for markdown::Block that allows deserialization
#[derive(Debug, Clone, Serialize)]
#[serde(try_from = "markdown::Block", into = "Block")]
pub(crate) struct Block(markdown::Block);
impl From<Block> for markdown::Block {
    fn from(block: Block) -> Self {
        block.0
    }
}
impl TryFrom<markdown::Block> for Block {
    type Error = ();
    fn try_from(value: markdown::Block) -> Result<Self, Self::Error> {
        Ok(Block(value))
    }
}

// newtype struct for markdown::Span that allows deserialization
#[derive(Debug, Clone, Serialize)]
#[serde(try_from = "markdown::Span", into = "Span")]
pub(crate) struct Span(markdown::Span);
impl From<Span> for markdown::Span {
    fn from(span: Span) -> Self {
        span.0
    }
}
impl TryFrom<markdown::Span> for Span {
    type Error = ();
    fn try_from(value: markdown::Span) -> Result<Self, Self::Error> {
        Ok(Span(value))
    }
}

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
                let mut styles : Vec<Style> = Vec::new();
                styles.push(Style::default());
                return styles;
            }
        };
    }

}

#[derive(Debug, Clone, Serialize)]
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

use std::io;
use mdbook::renderer::RenderContext;
fn main() {
    let styles = Style::get_styles();
    println!("{:#?}", styles);

    let mut stdin = io::stdin();
    
    let ctx = RenderContext::from_json(&mut stdin).unwrap();


    println!("{:?}", ctx);
}