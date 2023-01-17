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
#[derive(Serialize, Default, Deserialize, Debug)]
struct Style {
    // the name that the style is refrenced by in a section configuration
    #[serde(default = "Style::default_alias")]
    alias: String,
    // the font family for the style
    #[serde(default = "Style::default_font")]
    font: String,
    // the font size for the style
    #[serde(default = "Style::default_size")]
    size: u32,
    // the font color for the style
    #[serde(default = "Style::default_color")]
    color: String,
}

// implement default values for the Style struct
impl Style {

    // default function used by serde to populate the default value
    // for the default field when not defined in the Styles.toml file
    fn default_new() -> Style {
        Style {
            alias: Style::default_alias(),
            font: Style::default_font(),
            size: Style::default_size(),
            color: Style::default_color(),
        }
    }
    // default function used by serde to populate the default value
    // for the alias field when not defined in the Styles.toml file
    fn default_alias() -> String { "default".to_string() }
    // default function used by serde to populate the default value
    // for the font field when not defined in the Styles.toml file
    fn default_font() -> String { "sans-serif".to_string() }
    // default function used by serde to populate the default value
    // for the size field when not defined in the Styles.toml file
    fn default_size() -> u32 { 12 }
    // default function used by serde to populate the default value
    // for the color field when not defined in the Styles.toml file
    fn default_color() -> String { "#000000".to_string() }
}

// A Styles configuration including a default with optional additional
// style definitions. Values that are not defined for a specific style 
// are populated from the default style. If the default style has 
// missing values then the default values defined on the Style struct
// are used.
#[derive(Debug, Default, Serialize, Deserialize)]
struct Styles {
    // an individual default style that is used to populate missing values in all other styles
    #[serde(default = "Style::default_new")]
    default: Style,
    // a vector of styles that can be referenced by alias in a section configuration

    #[serde(default = "Styles::empty_styles")]
    style: Vec<Style>,
}
impl Styles {

    // default function used by serde to populate the default value
    fn empty_styles() -> Vec<Style> { Vec::new() }

    // looks for a Styles.toml file in the current directory and 
    // deserialises the file into a vector of Style structs
    fn get_styles() -> Styles {

        // current directory
        let current_dir = std::env::current_dir()
            .expect("Failed to identify the current directory.");
        let src_dir = current_dir
            .parent()
            .expect("Failed to parse the parent directory.")
            .to_str()
            .expect("Failed to parse the parent directory to a string.");

        let path = format!("{}/Styles.toml", src_dir);
        // debug print the path
        println!("Styles file: {}", path);

        // read an existing Styles.toml or create it and populate it with the default style
        match std::fs::read_to_string(path) {
            Ok(s) => {
                return toml::from_str(&s)
                    .expect("Failed to parse Styles.toml - check the file is valid TOML.");
            },
            Err(_) => {
                return Styles::default();
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
    let styles: Styles = Styles::get_styles();
    println!("{:#?}", styles);

    let mut stdin = io::stdin();
    
    let ctx = RenderContext::from_json(&mut stdin).unwrap();


    println!("{:#?}", ctx);
}