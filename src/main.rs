use anyhow::Result;
use log::error;
use mdbook::renderer::RenderContext;
use std::io::{self};

mod logs;
mod document;
mod pandoc;

fn run() -> Result<()> {
    let mut stdin = io::stdin();
    let ctx = RenderContext::from_json(&mut stdin)
        .expect("Error with the input data. Is everything formatted correctly?");

    let list: document::DocumentList = ctx
        .config
        .get_deserialized_opt("output.docx")
        .expect("Error reading \"output.docx\" configuration in book.toml. Check that all values are of the correct data type.")
        .unwrap_or_default();

    list.process(ctx.clone())
}

fn main() {
    logs::init_logger();

    let r = run();
    if let Err(e) = r {
        error!("An error has occurred while creating the document.\n{}", e);
    }
}
