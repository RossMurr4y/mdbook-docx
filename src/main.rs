extern crate docx_rs;
extern crate serde;
extern crate derive_more;
extern crate mdbook;

use docx_rs::Document;
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
    styles: Vec<Style>,
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

#[derive(Debug, Clone, Serialize)]
struct DocumentConfig {
    // the filename of the document to be produced by this
    // configuration set.
    filename: String,
    // a vector of file patterns / globs used to filter the input
    // markdown files to a subset. By default it matches everything.
    // note: any Sections listed on a DocumentConfig may have their
    // own includes attribute. Those will further filter the matching
    // values here. Patterns specified on a child Section that do
    // not also match the DocumentConfig patterns will be skipped.
    // If using Sections it is recommended to use a broader filter
    // at this level for this reason.
    #[serde(default = "DocumentConfig::default_includes")]
    includes: Vec<String>,
    // a vector of optional child document Sections which require
    // alternative Styles. Each part of your final document that
    // requires a different style to your defined "default" should
    // have its own Section with corresponding includes patterns.
    // Markdown content that does not match any includes pattern
    // on any of the Sections will receive your "default" style.
    #[serde(default = "DocumentConfig::default_sections")]
    sections: Vec<Section>
}
impl DocumentConfig {
    // the default function for the includes attribute when none is
    // defined. Returns a Vector of 1 pattern that when converted
    // later to a glob, will match all markdown content in the
    // RenderContext's book.sections
    fn default_includes() -> Vec<String> { vec!["*".to_string()] }

    // the default function for the sections attribute when none is
    // defined. Returns an empty vector.
    fn default_sections() -> Vec<Section> { vec![] }
}

// newtype struct for mdbook::renderer::RenderContext.
// this is necessary so that new methods can be added to RenderContext
// that allows direct conversion to/from RenderContext and
// DocumentConfig.
#[derive(Debug, Clone, Serialize)]
#[serde(try_from = "mdbook::renderer::RenderContext", into = "RenderContext")]
pub(crate) struct RenderContext(mdbook::renderer::RenderContext);
impl From<RenderContext> for mdbook::renderer::RenderContext {
    fn from(ctx: RenderContext) -> Self {
        ctx.0
    }
}
impl TryFrom<mdbook::renderer::RenderContext> for RenderContext {
    type Error = ();
    fn try_from(value: mdbook::renderer::RenderContext) -> Result<Self, Self::Error> {
        Ok(RenderContext(value))
    }
}
impl From<RenderContext> for DocumentConfig {
    fn from(ctx: RenderContext) -> Self {
        todo!("implement code to turn DocumentConfig into RenderContext")
    }
}
impl TryFrom<DocumentConfig> for RenderContext {
    type Error = ();
    fn try_from(value: DocumentConfig) -> Result<Self, Self::Error> {
        let r = todo!("implement code to turn RenderContext into Ok(DocumentConfig)");
        Ok(r)
    }
}

use std::io;
fn main() {
    let styles: Styles = Styles::get_styles();
    println!("{:#?}", styles);

    let mut stdin = io::stdin();
    
    let ctx = mdbook::renderer::RenderContext::from_json(&mut stdin)
        .expect("Failure to create RenderContext. Check the syntax of Book.toml is correct.");


    println!("{:#?}", ctx);
}