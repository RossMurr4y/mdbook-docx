// A text formatting style definition.
struct Style {
    // the name that the style is refrenced by in a section configuration
    alias: String,
    // the font family for the style
    font: String,
    // the font size for the style
    size: u32,
    // the font color for the style
    color: String,
}


fn main() {
    println!("Hello, world!");
}