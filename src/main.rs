use anyhow::{bail, Result};
use chrono::Local;
use env_logger::Builder;
use glob::Pattern;
use log::{error, LevelFilter};
use mdbook::renderer::RenderContext;
use mdbook::BookItem;
use pandoc::{OutputKind, MarkdownExtension, Pandoc, PandocOption};
use serde_derive::Deserialize;
use std::{
    env,
    io::{self, Write},
    path::PathBuf,
};

fn init_logger() {
    let mut builder = Builder::new();

    builder.format(|formatter, record| {
        writeln!(
            formatter,
            "{} [{}] ({}): {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            record.level(),
            record.target(),
            record.args()
        )
    });

    if let Ok(var) = env::var("RUST_LOG") {
        builder.parse_filters(&var);
    } else {
        // if no RUST_LOG provided, default to logging at the Info level
        builder.filter(None, LevelFilter::Info);
        // Filter extraneous html5ever not-implemented messages
        builder.filter(Some("html5ever"), LevelFilter::Error);
    }

    builder.init();
}

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
    fn process(self, context: RenderContext) -> Result<()> {
        for doc in self.documents {
            let result = doc.process(context.clone());
            // if this itteration has an error, return the error
            // else loop continues
            if let Err(e) = result {
                bail!(e)
            };
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
    #[serde(default)]
    pub append: Option<Vec<PathBuf>>,
    #[serde(default)]
    pub prepend: Option<Vec<PathBuf>>,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            filename: PathBuf::from("output.docx".to_string()),
            template: Some(PathBuf::from("reference.docx".to_string())),
            include: Some(vec![PathBuf::from("*".to_string())]),
            offset_headings_by: None,
            append: None,
            prepend: None,
        }
    }
}

impl Document {
    fn get_chapters(&self, context: &RenderContext) -> Result<Vec<PathBuf>> {
        // valid globs
        let patterns = self.get_patterns()?;

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
        if ch.is_empty() {
            bail!("No markdown files match the specified include and/or exclude filters. Verify your filenames and filters are correct.")
        };
        Ok(ch)
    }

    // establish the list of globs based on the list of includes
    fn get_patterns(&self) -> Result<Vec<Pattern>> {
        let mut patterns: Vec<Pattern> = Vec::new();
        for buf in self.include.clone().unwrap_or_default() {
            patterns.push(
                Pattern::new(buf.to_str().unwrap())
                    .expect("Unable to create Pattern from provided include."),
            );
        }
        // if patterns remains empty, use wildcard catch-all glob
        if patterns.len() == 0 {
            println!("No include value provided. Using wildcard glob.");
            patterns.push(Pattern::new("*").expect("Error using wildcard glob."));
        }
        if patterns.is_empty() {
            bail!("No files matched the provided include / exclude filters.");
        };
        Ok(patterns)
    }

    // filter the book content based on include/exclude values
    fn get_filtered_content(&self, context: &RenderContext) -> Result<String> {
        let mut content = String::new();
        let chapters = self.get_chapters(context)?;

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
        if content.is_empty() {
            bail!("The provided include/exclude filters do not match any content.");
        };
        Ok(content)
    }

    fn process(self, context: RenderContext) -> Result<()> {
        // get the static, non-configurable pandoc configuration
        let mut pandoc_config = PandocConfig::default();
        pandoc_config.assign_options(&context, &self);

        // set the content
        let content = self.get_filtered_content(&context)?;

        let mut pandoc = Pandoc::new();
        pandoc.add_options(pandoc_config.options.as_slice());

        pandoc.set_input_format(
            pandoc::InputFormat::MarkdownGithub,
            pandoc_config.input_extensions,
        );
        pandoc.set_input(pandoc::InputKind::Pipe(content.to_string()));
        pandoc.set_output_format(pandoc::OutputFormat::Docx, pandoc_config.output_extensions);
        pandoc.set_output(OutputKind::File(self.filename.clone()));

        pandoc.set_show_cmdline(true);

        // execute the pandoc cli and bail on an error
        // If pandoc errored, present the error in our DocumentError
        if let Err(e) = pandoc.execute() {
            bail!(e);
        }

        // now the markdown > docx has completed, combine it
        // with any append/prepend files if specified
        self.combine_sections(&context)?;

        Ok(())
    }

    fn combine_sections(self, context: &RenderContext) -> Result<()> {
        let mut parts: Vec<PathBuf> = vec![];

        // init our parts list with the prepends
        if let Some(paths) = &self.prepend {
            for p in paths {
                parts.push(context.root.clone().join(p))
            }
        };
        // add our main file to the vec
        parts.push(context.root.clone().join("book/docx").join(self.filename.clone()));
        // complete our parts list with any appends
        if let Some(paths) = &self.append {
            for p in paths {
                parts.push(context.root.clone().join(p))
            }
        };

        // have pandoc merge them
        let mut pandoc = Pandoc::new();
        pandoc.set_output(OutputKind::File(self.filename.clone()));
        for part in parts {
            pandoc.add_input(&part);
        };

        let mut config = PandocConfig::default();
        config.assign_options(context, &self);
        pandoc.add_options(config.options.as_slice());
        pandoc.execute()?;

        Ok(())
    }
}

pub struct PandocConfig {
    pub input_extensions: Vec<MarkdownExtension>,
    pub output_extensions: Vec<MarkdownExtension>,
    pub options: Vec<PandocOption>,
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
            options: vec![],
            content: Default::default(),
        }
    }
}

impl PandocConfig {
    // Assigns the required list of pandoc options to the PandocConfig
    // so later they can be set on the Pandoc struct
    fn assign_options(&mut self, context: &RenderContext, doc: &Document) -> &Self {
        // directory of book.toml and root of where we'll look for content
        let data_dir = context.root.clone();
        // path of the src directory
        let src_path = PathBuf::from(&data_dir).join("src");
        self.options = vec![
            PandocOption::DataDir(data_dir.clone()),
            PandocOption::ResourcePath(vec![src_path]),
            PandocOption::AtxHeaders,
            PandocOption::ReferenceLinks,
        ];

        // set the shift-heading-level option if specified
        if let Some(i) = doc.offset_headings_by {
            self.options.push(PandocOption::ShiftHeadingLevelBy(i))
        };
        // set the reference template if specified
        if let Some(path) = &doc.template {
            self.options.push(PandocOption::ReferenceDoc(data_dir.clone().join(path)))
        };
        // set the file to prepend to the document if specified
        if let Some(path) = &doc.prepend {
            for p in path {
                self.options.push(PandocOption::IncludeBeforeBody(data_dir.clone().join(p.to_owned())))
            }
        };
        // set the file(s) to append to the document if specified
        if let Some(path) = &doc.append {
            for p in path {
                self.options.push(PandocOption::IncludeAfterBody(data_dir.join(p.to_owned())))
            }
        };

        self
    }
}

fn run() -> Result<()> {
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
    init_logger();

    let r = run();
    if let Err(e) = r {
        error!("An error has occurred while creating the document.\n{}", e);
    }
}
