# mdbook-docx

A Docx backend for [mdBook](https://rust-lang.github.io/mdBook/).

`mdbook-docx` translates your mdBook `.md` (Markdown) files into Word `.docx` documents.

Styling of the content is achieved with a template `.docx` file. The content of the template is ignored but the **style definitions** are used to construct the final document. 

[Example](./examples/reference.docx)


## Usage

If you're new to mdBook, [get started here](https://rust-lang.github.io/mdBook/guide/installation.html).

Once you've a working mdBook, update your `book.toml` with the docx backend.

If you've only had the default html backend until now and you want to now produce both the HTML and Docx content, you'll want to add both to continue building both.

```toml
[book]
title = "Example Book"
authors = ["Your Name"]

[output.html]

[output.docx]
[[output.docx.documents]]
filename = "example-output.docx"
```

Build your book to output your document(s) alongside the other backend outputs in `book/docx/`

```terminal
mdbook build
```

## Installation

> `mdbook-docx` is currently in heavy development and is not yet available with cargo.
> Recommended use for now is with [Docker](#docker).

Install `mdbook-docx` binary from local clone

```bash
cargo install --path .
```

Add the following to your `book.toml` to enable the Docx backend.

The mandatory and default settings are provided below.

```bash
[output.docx]
```

### Docker

There are two primary docker images published, based on your needs:

- `rossmurr4y/mdbook-docx` contains only `mdBook` and `mdbook-docx`
- `rossmurr4y/mdbook` includes a number of additional backends as well as includes laTex and pdf-creation tooling, however it is much larger.

```terminal
# update your volume mount path as necessary.
# make sure all content referenced by your .md files is inside the container.
docker run -it --volume $(pwd):/book rossmurr4y/mdbook build
```

## Configuration

The configuration accepts an array of `Documents`. At least one must be specified, however you can create multiple documents from the once source.

### Mandatory

```toml
...

[output.docx]
[[output.docx.documents]]
```

### Defaults

```toml
...

[output.docx]
[[output.docx.documents]]
filename = "output.docx"
template = "reference.docx"
include = ["*"]
offset_headings_by = 0
append = []
prepend = []
```

| configuration | description | valid values |
| ------------- | ----------- | ------------ |
| filename | the name of the file output to produce, including extension | `string` Defaults to `output.docx` |
| template | path to a template file to use for styling only | `string` File path relative to your book.toml |
| include | An array of paths to include in the document output. Allows for Unix shell style patterns/globs. | `string[]` Relative to your `./src` dir. Files must be present in your SUMMARY.md |
| offset_headings_by | By default, Markdown H1's become the `title` style type, and Markdown H2's translate to Docx H1's. This allows you to adjust this by shifting the Markdown H1s up/down as desired. | `int` -1, or 1 will usually do |
| append | An array of filepaths to sequentially append to the end of the generated file. Styles will be adjusted to match the primary output. | `string[]` Paths relative to your book.toml |
| prepend | An array of filepaths to sequentially prepend to the start of the generated file. Styles will be adjusted to match the primary output. | `string[]` Paths relative to your book.toml |

### Examples

```toml
...
# adding the docx backend
[output.docx]

# document created from all files
[[output.docx.documents]]
filename = "CompleteGuide.docx"

# just a subset of files using explicit file names
[[output.docx.documents]]
filename = "DeploymentGuide.docx"
include = ["intro.md", "deployment_guide.md", "appendix_deployment_01.md"]

# specific style template with globs and filenames
[[output.docx.documents]]
filename = "TechnicalDesignGuide.docx"
template = "TechnicalStyles.docx"
include = ["intro.md", "tech_*.md", "reference_tables*.md", "**/appendix*tech*.md"]

# - a specific title page and some legal-ese
# - followed by the Markdown content
# - followed by glossary of terms maintained elsewhere
[[output.docx.documents]]
filename = "ImportantReport.docx"
template = "ReportStyles.docx"
include = ["intro.md", "report_*.md", "charts*.md", "**/appendix*report*.md"]
prepend = ["title-page.docx", "legal-template.docx"]
append = [ "glossary.docx" ]
```
