use std::{io, fmt, error::Error, path::PathBuf};
use mdbook::{BookItem};
use mdbook::renderer::RenderContext;
use pandoc::{Pandoc,MarkdownExtension, PandocOption, PandocError};
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

impl DocumentList {
    fn process(self, context: RenderContext) -> Result<(), DocumentError> {

        for doc in self.documents {
            let result = doc.process(context.clone());
            // if this itteration has an error, return the error
            // else loop continues
            if let Err(e) = result { return Err(e) };
        }

        Ok(())
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
        //println!("Filtered content: {:#?}", &chapters);

        for item in context.book.iter() {
            if let BookItem::Chapter(ref ch) = *item {
                if let true = &ch.path.is_some() {
                    if let true = chapters.contains(&ch.path.clone().unwrap()) {
                        content.push_str(&ch.content);
                        // chapter content in mdBook strips out newlines at the end of a file.
                        // because we want to play it safe and add the MarkdownExtension
                        // BlankBeforeHeader by default, this prevents all the level-1 headers
                        // - h1's - from being accepted as headers, so they come out styled incorrectly.
                        // To resolve this we simply append two newlines to the end of every chapter.
                        content.push_str("\n\n");
                    }
                }
            }
        }
        
        // return content
        content
    }

    fn process(self, context: RenderContext) -> Result<(), DocumentError> {
        // get the static, non-configurable pandoc configuration
        let pandoc_config = PandocConfig::default();

        // set the content
        let content = self.get_filtered_content(&context);

        let mut pandoc = Pandoc::new();
        pandoc.set_input_format(pandoc::InputFormat::MarkdownGithub, pandoc_config.input_extensions);
        pandoc.set_input(pandoc::InputKind::Pipe(content.to_string()));
        pandoc.set_output_format(pandoc::OutputFormat::Docx, pandoc_config.output_extensions);
        pandoc.set_output(pandoc::OutputKind::File(self.filename));

        // set pandoc options
        let src_path = PathBuf::from(&context.root).join("src");
        pandoc.add_option(pandoc::PandocOption::DataDir(context.root.clone()));
        pandoc.add_option(pandoc::PandocOption::ResourcePath(vec!(src_path.clone())));
        pandoc.add_option(pandoc::PandocOption::AtxHeaders);
        pandoc.add_option(pandoc::PandocOption::ReferenceLinks);

        // if a heading offset was specified in the config, use it
        if let Some(o) = self.offset_headings_by { pandoc.add_option(pandoc::PandocOption::ShiftHeadingLevelBy(o)); }
        // if a template was specified in the config, use it
        if let Some(t) = self.template { pandoc.add_option(PandocOption::ReferenceDoc(PathBuf::from(context.root).join(t))); }
        
        // output the pandoc cmd for debugging
        pandoc.set_show_cmdline(true);

        let result = pandoc.execute();

        // If pandoc errored, present the error in our DocumentError
        if let Err(e) = result {
            return Err(DocumentError::PandocExecutionError(e))
        }

        Ok(())
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
                MarkdownExtension::BlankBeforeHeader,
                MarkdownExtension::TableCaptions,
                MarkdownExtension::PandocTitleBlock,
                MarkdownExtension::YamlMetadataBlock,
                MarkdownExtension::ImplicitHeaderReferences,
            ], 
            output_extensions: vec![],
            content: Default::default() 
        }
    }
}


fn run() -> Result<(), DocumentError> {
    let mut stdin = io::stdin();
    let ctx = RenderContext::from_json(&mut stdin)
        .expect("Error with the input data. Is everything formatted correctly?");

    let list: DocumentList = ctx
        .config
        .get_deserialized_opt("output.docx")
        .expect("Error reading \"output.docx\" configuration in book.toml. Check that all values are of the correct data type.")
        .unwrap_or_default();

    //println!("List of documents: {:#?}", list.documents);

    // loop over each document configuration and create it
    // Return the result
    list.process(ctx.clone())
}


fn main() {

    let r = run();
    if let Err(e) = r {
        panic!("\n\nAn error has occurred while creating the document.\n{}", e);
    }

}

#[derive(Debug)]
pub enum DocumentError {
    PandocExecutionError(pandoc::PandocError),
}

impl Error for DocumentError {}

impl fmt::Display for DocumentError {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        use DocumentError::*;
        match &*self {
            PandocExecutionError(e) => { DocumentError::process_pandoc_err(e)},
        }
        Ok(())
    }
}

impl DocumentError {
    fn process_pandoc_err(error: &PandocError){
        match error {
            PandocError::BadUtf8Conversion(_) => { eprintln!("\n[ERROR]\tThe reference template is not valid UTF-8.") },
            PandocError::IoErr(e) => { eprintln!("\n[ERROR]\tError reading or writing file. Details: {}", e) },
            PandocError::Err(_e) => { eprintln!("\n[ERROR]\tAn input file could not be found.") },
            PandocError::NoOutputSpecified => { eprintln!("\n[ERROR]\tNo output file has been specified.") },
            PandocError::NoInputSpecified => { eprintln!("\n[ERROR]\tNo input has been provided.") },
            PandocError::PandocNotFound => { eprintln!("\n[ERROR]\tPandoc not found. Check that pandoc is installed, and available on the $PATH.") },
        }
    }
}