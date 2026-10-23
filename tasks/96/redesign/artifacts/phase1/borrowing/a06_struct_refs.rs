// A6: a struct holding a reference needs a lifetime parameter, and it spreads.
pub struct Lexer { src: &str, pos: usize }
