use mdbook::renderer::RenderContext;
use pandoc::{MarkdownExtension, PandocOption};
use std::path::PathBuf;
use crate::document::{Document, DocumentContent};

pub struct PandocConfig {
    pub input_extensions: Vec<MarkdownExtension>,
    pub output_extensions: Vec<MarkdownExtension>,
    pub options: Vec<PandocOption>,
    pub content: DocumentContent,
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
    pub fn assign_options(&mut self, context: &RenderContext, doc: &Document) -> &Self {
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