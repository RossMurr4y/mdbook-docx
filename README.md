# mdbook-docx

A Docx backend for [mdBook](https://rust-lang.github.io/mdBook/)

## Usage

Update your `book.toml` with the docx backend.

If you've only had the default html backend until now and you want to now produce both the HTML and Docx content, you should add both.

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

## Configuration

The configuration accepts an array of `Documents`. At least one must be specified.

### Mandatory

```toml
...

[output.docx]
[[output.docx.documents]]
filename = "example-output.docx"
```

### Defaults

```toml
...

[output.docx]
[[output.docx.documents]]
filename = "output.docx"
template = "reference.docx"
include = ["*"]
```

| configuration | description | valid values |
| ------------- | ----------- | ------------ |
| filename | the name of the file output to produce, including extension | |
| template | path to a template file to use for styling only | Path starts at the same level as the book.toml |
| include | An array of paths to include in the document output. Allows for Unix shell style patterns/globs. | Paths start within the books src dir. They must be present in your SUMMARY.md file (or added to the book via other pre-processor) or they will not be included. |

### Example

```toml
...
# adding the docx backend
[output.docx]

# document created from all files
[[output.docx.documents]]
filename = "CompleteGuide.docx"

# document created from just a subset of files, using explicit file names
[[output.docx.documents]]
filename = "DeploymentGuide.docx"
include = ["intro.md", "deployment_guide.md", "appendix_deployment_01.md"]

# document created with a specific style template, with globs and filenames
[[output.docx.documents]]
filename = "TechnicalDesignGuide.docx"
template = "TechnicalStyles.docx"
include = ["intro.md", "tech_*.md", "reference_tables*.md", "**/appendix*tech*.md"]
```
