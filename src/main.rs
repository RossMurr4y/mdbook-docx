extern crate docx_rs;
extern crate serde;
extern crate derive_more;
extern crate mdbook;
extern crate glob;

// newtype struct for markdown::ListItem that allows deserialization
use serde::{Serialize, Deserialize};
use derive_more::From;

#[derive(Debug, Clone, From, Deserialize)]
#[serde(try_from = "ListItem", into = "markdown::ListItem")]
pub(crate) struct ListItem(markdown::ListItem);

// newtype struct for markdown::Block that allows deserialization
#[derive(Debug, Clone, From, Deserialize)]
#[serde(try_from = "Block", into = "markdown::Block")]
pub(crate) struct Block(markdown::Block);

// newtype struct for markdown::Span that allows deserialization
#[derive(Debug, Clone, From, Deserialize)]
#[serde(try_from = "Span", into = "markdown::Span")]
pub(crate) struct Span(markdown::Span);

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

#[derive(Debug, Clone, Deserialize)]
struct Section {
    // A block of tokenized markdown content.
    block: Block,
    // The style alias that the corresponding Block should be formatted in.
    style: String,
}

// the section configuration struct as represented in the book.toml
// configuration file. This struct is how end users will associate
// sub-sections of their final document along with particular styles.
#[derive(Debug, Clone, From, Deserialize)]
struct SectionConfig {
    // the alias to a pre-existing style definition. Style definitions
    // should be provided by way of a Styles.toml configuration file
    // or one of the plugin-provided "out-of-the-box" defintions.
    style: String,
    // The markdown file globs to which define this sections content.
    // Blocks from files that match these globs will be added to this section
    // and be formatted with the style alias defined by the style field.
    includes: Vec<String>,
}

#[derive(Debug, Clone, From, Deserialize)]
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
    sections: Vec<SectionConfig>
}
impl DocumentConfig {
    // the default function for the includes attribute when none is
    // defined. Returns a Vector of 1 pattern that when converted
    // later to a glob, will match all markdown content in the
    // RenderContext's book.sections
    fn default_includes() -> Vec<String> { vec!["*".to_string()] }

    // the default function for the sections attribute when none is
    // defined. Returns an empty vector.
    fn default_sections() -> Vec<SectionConfig> { vec![] }

    // builder for initializing a new DocumentConfig.
    fn new(ctx: &RenderContext) -> DocumentConfig {
        ctx
            .clone()
            .config
            .get_deserialized_opt("output.docx")
            .expect("Invalid book.toml config. Check the values of [output.docx]")
            .unwrap() // safe unwrap due to expect
    }
}

// newtype struct for mdbook::renderer::RenderContext.
// this is necessary so that new methods can be added to RenderContext
// that allows direct conversion to/from RenderContext and
// DocumentConfig.
#[derive(Debug, Clone, From, Serialize)]
#[serde(try_from = "mdbook::renderer::RenderContext", into = "RenderContextDef")]
pub(crate) struct RenderContextDef(mdbook::renderer::RenderContext);
impl From<RenderContextDef> for mdbook::renderer::RenderContext {
    fn from(ctx: RenderContextDef) -> Self {
        ctx.0
    }
}


use std::io;
use std::ops::Deref;
use std::path::PathBuf;
use mdbook::renderer::{RenderContext};
use mdbook::book::{Chapter, BookItem};
use glob::Pattern;
fn main() -> Result<(), std::fmt::Error> {
    let styles: Styles = Styles::get_styles();
    println!("{:#?}", styles);

    let mut stdin = io::stdin();
    
    let ctx: RenderContext = RenderContext::from_json(&mut stdin)
        .expect("Invalid book.toml config.");

    let mut cfg = DocumentConfig::new(&ctx);

    // filepath root for content as per the book.toml
    let path_root: PathBuf = ctx.clone().root;

    // initialize a vector to hold new Sections
    let mut sections: Vec<Section> = vec![];
    
    // loop over all the remaining BookItems, for each:
    for item in ctx.clone().book.iter() {
        if let BookItem::Chapter(ref ch) = *item {
            if !ch.is_draft_chapter() {
                // if the Chapter path matches any of the includes globs from DocumentConfig scope
                if cfg.clone()
                    .includes
                    .into_iter()
                    .any(|x| {
                        let p = Pattern::new(x.as_str()).unwrap();
                        let path = ch
                            .path
                            .as_ref()
                            .expect("Unable to identify chapter path.");
                        p.matches(path.to_str().unwrap())
                    }) {

                        // at least one of the Patterns match, so include chapter
                        // if it matches a SectionConfig pattern, use that SectionConfig
                        // to construct this new Section. Else use use default style.
                        let sec_cfg: Vec<SectionConfig> = cfg.clone()
                            .sections
                            .into_iter()
                            .filter(|s| {
                                s.includes
                                    .iter()
                                    .any(|x| {
                                        let p = Pattern::new(x.as_str()).unwrap();
                                        let path = ch
                                            .path
                                            .as_ref()
                                            .expect("Unable to identify chapter path from Section config.");
                                        p.matches(path.to_str().unwrap())
                                    })
                            }).collect();

                        println!("chapter: {:#?},  sec_cfg: {:#?}", ch.path, sec_cfg);
                        
                        // tokenize the markdown content for this section
                        let sec_blocks = markdown::tokenize(ch.content.as_str());

                        // if the filtered section config is empty, then the current chapter
                        // should be style with the default style. Otherwise with the
                        // SectionConfig style
                        let sec_style: String = match sec_cfg.is_empty() {
                            true => "default".to_string(),
                            false => sec_cfg.first().unwrap().style.clone(),
                        };
                        // then for each block, push up a new section into our final document sections
                        // vector with the style matching the current SectionConfig
                        for block in sec_blocks.into_iter() {
                            sections.push(Section {
                                style: sec_style.clone(),
                                block: block.into(),
                            })
                        }
                    }
            }
        }
    }

    println!("sections: {:#?}", sections);


    //     evaluate its path against the SectionConfig globs.

    //     if it matches a glob, produce a new Section
    //          with its Block containing the content
    //          and its Style set to the alias of the SectionConfig
    
    //     else
    //          produce a new section
    //          with its block containing the content
    //          and its style set to the default alias

    // complete
    Ok(())
}