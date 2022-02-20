use std::{io, path::PathBuf};
use mdbook::{BookItem};
use mdbook::renderer::RenderContext;
use pandoc::{Pandoc,MarkdownExtension, PandocOption};
use serde_derive::{Deserialize};
use glob::Pattern;

#[derive(Debug, Deserialize)]
pub struct DocumentList {
    #[serde(default)]
    documents: Vec<Document>,
}

impl Default for DocumentList {
    fn default() -> Self {
        Self { documents: vec![] }
    }
}

#[derive(Debug, Deserialize)]
pub struct Document {
    #[serde(default)]
    pub filename: PathBuf,
    #[serde(default)]
    pub template: Option<PathBuf>,
    #[serde(default)]
    pub include: Option<Vec<PathBuf>>,
    #[serde(default)]
    pub offset_headings_by: Option<i32>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            filename: PathBuf::from("output.docx".to_string()),
            template: Some(PathBuf::from("reference.docx".to_string())),
            include: Some(vec![PathBuf::from("*".to_string())]),
            offset_headings_by: None,
        }
    }
}

impl Document {

    fn get_chapters(&self, context: &RenderContext) -> Vec<PathBuf> {

        // valid globs
        let patterns = self.get_patterns();

        let mut ch: Vec<PathBuf> = Vec::new();
        for item in context.book.iter() {
            if let BookItem::Chapter(ref c) = *item {
                if c.path.is_some() {

                    let mut pattern_match = false;
                    for p in &patterns {
                        if p.matches_path(c.path.clone().unwrap().as_path()) {
                            pattern_match = true;
                        }
                    }
                    if pattern_match == true {
                        ch.push(c.path.clone().unwrap())
                    }
                }
            }
        }
        ch
    }

    // establish the list of globs based on the list of includes
    fn get_patterns(&self) -> Vec<Pattern> {
        let mut patterns: Vec<Pattern> = Vec::new();
        for buf in self.include.clone().unwrap_or_default() {
            patterns.push(Pattern::new(buf.to_str().unwrap()).expect("Unable to create Pattern from provided include."));
        }
        // if patterns remains empty, use wildcard catch-all glob
        if patterns.len() == 0 {
            println!("No include value provided. Using wildcard glob.");
            patterns.push(Pattern::new("*").expect("Error using wildcard glob."));
        }
        patterns
    }

    // filter the book content based on include/exclude values
    fn get_filtered_content(&self, context: &RenderContext) -> String {
        
        let mut content = String::new();
        let chapters = self.get_chapters(context);
        println!("Filtered content: {:#?}", &chapters);

        for item in context.book.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                if let true = &ch.path.is_some() {
                    if let true = chapters.contains(&ch.path.clone().unwrap()) {
                        content.push_str(&ch.content);
                    }
                }
            }
        }
        
        // return content
        content
    }

}

pub struct PandocConfig {
    pub input_extensions: Vec<MarkdownExtension>,
    pub output_extensions: Vec<MarkdownExtension>,
    pub content: String,
}

impl Default for PandocConfig {
    fn default() -> Self {
        Self { 
            input_extensions: vec![
                MarkdownExtension::PipeTables, 
                MarkdownExtension::RawHtml, 
                MarkdownExtension::AutolinkBareUris, 
                MarkdownExtension::AutoIdentifiers, 
                MarkdownExtension::HardLineBreaks, 
                MarkdownExtension::BlankBeforeHeader
            ], 
            output_extensions: vec![],
            content: Default::default() 
        }
    }
}



fn main() {

    let mut stdin = io::stdin();
    let ctx = RenderContext::from_json(&mut stdin)
        .expect("Error with the input data. Is everything formmated correctly?");

    let list: DocumentList = ctx
        .config
        .get_deserialized_opt("output.docx")
        .expect("Error reading \"output.docx\" configuration in book.toml. Check that all values are of the correct data type.")
        .unwrap_or_default();

    println!("List of documents: {:#?}", list.documents);

    // loop over each document configuration and create it
    for doc in list.documents {

        let context = ctx.clone();

        // get the static, non-configurable pandoc configuration
        let pandoc_config = PandocConfig::default();

        // set the content
        let content = doc.get_filtered_content(&context);

        let mut pandoc = Pandoc::new();
        pandoc.set_input_format(pandoc::InputFormat::MarkdownGithub, pandoc_config.input_extensions);
        pandoc.set_input(pandoc::InputKind::Pipe(content.to_string()));
        pandoc.set_output_format(pandoc::OutputFormat::Docx, pandoc_config.output_extensions);
        pandoc.set_output(pandoc::OutputKind::File(doc.filename));

        // set pandoc options
        let src_path = PathBuf::from(&context.root).join("src");
        pandoc.add_option(pandoc::PandocOption::DataDir(ctx.root.clone()));
        pandoc.add_option(pandoc::PandocOption::ResourcePath(vec!(src_path.clone())));
        pandoc.add_option(pandoc::PandocOption::AtxHeaders);
        // if a heading offset was specified in the config, use it
        if let Some(o) = doc.offset_headings_by { pandoc.add_option(pandoc::PandocOption::ShiftHeadingLevelBy(o)); }
        // if a template was specified in the config, use it
        if let Some(t) = doc.template { pandoc.add_option(PandocOption::ReferenceDoc(PathBuf::from(context.root).join(t))); }
        
        // output the pandoc cmd for debugging
        pandoc.set_show_cmdline(true);

        pandoc.execute().expect("Cannot unwrap the result.");

    }
}
