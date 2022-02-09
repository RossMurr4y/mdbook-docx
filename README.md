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
filename = "example-output.docx"
```

Build your book to output your Docx alongside the other backend outputs in `book/docx/`

```terminal
mdbook build
```

## Configuration

### Mandatory

```toml
...

[output.docx]
filename = "example-output.docx"
```

### Defaults

```toml
...

[output.docx]
filename = "example-output.docx"
template = ""
```

| configuration | description | valid values |
| ------------- | ----------- | ------------ |
| filename | the name of the file output to produce, including extension | |
| template | path to a template file to use for styling only | Path starts at the same level as the book.toml | 

## Installation

TODO