use std::{io, path::PathBuf};
use mdbook::{BookItem};
use mdbook::renderer::RenderContext;
use pandoc::{Pandoc,MarkdownExtension, PandocOption};
use serde_derive::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct Document {
    pub filename: PathBuf,
    pub template: Option<PathBuf>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            filename: PathBuf::from("output.docx".to_string()),
            template: Some(PathBuf::from("reference.docx".to_string())),
        }
    }
}

pub struct PandocConfig {
    pub input_extensions: Vec<MarkdownExtension>,
    pub content: String,
}

impl Default for PandocConfig {
    fn default() -> Self {
        Self { 
            input_extensions: vec![MarkdownExtension::PipeTables, MarkdownExtension::ImplicitFigures], 
            content: Default::default() 
        }
    }
}



fn main() {

    let mut stdin = io::stdin();
    let ctx = RenderContext::from_json(&mut stdin)
        .expect("Error with the input data. Is everything formmated correctly?");

    let document: Document = ctx
        .config
        .get_deserialized_opt("output.docx")
        .expect("Error reading \"output.docx\" configuration in book.toml")
        .unwrap_or_default();

    // get the static, non-configurable pandoc configuration
    let pandoc_config = PandocConfig::default();

    // set the content
    let mut content = String::new();
    for item in ctx.book.iter() {
        if let BookItem::Chapter(ref ch) = *item {
            if let true = &ch.path.is_some() {
                content.push_str(&ch.content);
            }
        }
    }

    let mut pandoc = Pandoc::new();
    pandoc.set_input_format(pandoc::InputFormat::Commonmark, pandoc_config.input_extensions);
    pandoc.set_input(pandoc::InputKind::Pipe(content.to_string()));
    pandoc.set_output_format(pandoc::OutputFormat::Docx, vec!());
    pandoc.set_output(pandoc::OutputKind::File(document.filename));

    // set pandoc options
    let src_path = PathBuf::from(&ctx.root).join("src");
    pandoc.add_option(pandoc::PandocOption::DataDir(ctx.root.clone()));
    pandoc.add_option(pandoc::PandocOption::ResourcePath(vec!(src_path.clone())));
    // if a template was specified in the config, use it
    if let Some(t) = document.template { pandoc.add_option(PandocOption::ReferenceDoc(PathBuf::from(ctx.root).join(t))); }
    
    // output the pandoc cmd for debugging
    pandoc.set_show_cmdline(true);

    pandoc.execute().expect("Cannot unwrap the result.");
}
