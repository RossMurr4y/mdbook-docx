extern crate docx_rs;
extern crate serde;
extern crate derive_more;
extern crate mdbook;
extern crate glob;
extern crate zip;

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
    size: u64,
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
    fn default_size() -> u64 { 12 }
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

        let path = format!("{src_dir}/Styles.toml");

        // read an existing Styles.toml or create it and populate it with the default style
        match std::fs::read_to_string(path) {
            Ok(s) => {
                toml::from_str(&s)
                    .expect("Failed to parse Styles.toml - check the file is valid TOML.")
            },
            Err(_) => {
                Styles::default()
            }
        }
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
use mdbook::renderer::{RenderContext};
use mdbook::book::{BookItem};
use glob::Pattern;
use docx_rs::{Paragraph, Run, RunFonts, Docx};
fn main() -> zip::result::ZipResult<()> {

    let mut stdin = io::stdin();
    
    let ctx: RenderContext = RenderContext::from_json(&mut stdin)
        .expect("Invalid book.toml config.");

    let cfg = DocumentConfig::new(&ctx);

    // filepath root for content as per the book.toml
    let path_root = ctx.root.as_path();

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
                        
                        // tokenize the markdown content for this section
                        let sec_blocks = markdown::tokenize(ch.content.as_str());

                        // if the filtered section config is empty, then the current chapter
                        // should be style with the default style. Otherwise with the
                        // SectionConfig style
                        println!("sec_cfg: {sec_cfg:#?}");
                        let sec_style: String = match sec_cfg.is_empty() {
                            true => "default".to_string(),
                            false => sec_cfg.first().unwrap().style.clone(),
                        };
                        // then for each block, push up a new section into our final document sections
                        // vector with the style matching the current SectionConfig
                        for block in sec_blocks.into_iter() {
                            println!("sec_style: {sec_style:#?}");
                            sections.push(Section {
                                style: sec_style.clone(),
                                block: block.into(),
                            })
                        }
                    }
            }
        }
    }

    // init the output document
    let file = std::fs::File::create(&path_root.join(cfg.filename))
        .expect("Failed to initialize the output document.");
    let mut paragraphs: Vec<Paragraph> = vec![]; 

    let styles: Styles = Styles::get_styles();

    // loop over all the sections now and add them to an output document
    // with their stylings
    for sec in sections.into_iter() {

        // filter the available styles to just the one defined by the section
        let opt_style = &styles.styles
            .iter()
            .filter(|&s| s.alias == sec.style)
            .collect::<Vec<&Style>>();

        let default_style = Style::default_new();
        let sec_style: &Style = if opt_style.is_empty() {
            &default_style
        } else {
            opt_style.first().expect("Unable to find Style for section.").deref()
        };

        match sec.block.0 {
            markdown::Block::Header(_, _) => {println!("todo: header")},
            markdown::Block::Paragraph(p) => { 
                // init a new paragraph

                for span in p.into_iter() {
                    match span {
                        markdown::Span::Break => { println!("todo: break") },
                        markdown::Span::Text(t) => {

                            let para = Paragraph::new()
                                .add_run(
                                    Run::new()
                                        .add_text(t)
                                        .fonts(RunFonts::new().ascii(&sec_style.font))
                                        .size((sec_style.size.clone() * 2).try_into().expect("Failed to translate style size to usize."))
                                        .color(&sec_style.color)
                                    );
                            paragraphs.push(para);
                        },
                        markdown::Span::Code(_) => {println!("todo: code")},
                        markdown::Span::Link(_, _, _) => {println!("todo: link")},
                        markdown::Span::Image(_, _, _) => {println!("todo: image")},
                        markdown::Span::Emphasis(_) => {println!("todo: emphasis")},
                        markdown::Span::Strong(_) => {println!("todo: strong")},
                    }
                }
            },
            markdown::Block::Blockquote(_) => {println!("todo: blockquote")},
            markdown::Block::CodeBlock(_, _) => {println!("todo: codeblock")},
            markdown::Block::OrderedList(_, _) => {println!("todo: orderedlist")},
            markdown::Block::UnorderedList(_) => {println!("todo: unorderedlist")},
            markdown::Block::Raw(_) => {println!("todo: raw")},
            markdown::Block::Hr => {println!("todo: hr")},
        }
    }

    let mut doc = Docx::new();

    for para in paragraphs.into_iter() {
        doc = doc.add_paragraph(para);
    }

    doc.build()
        .pack(file)
}