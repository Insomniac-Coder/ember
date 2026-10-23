// A7: a document that owns its text and keeps slices of it.
pub struct Document<'a> { pub text: String, pub words: Vec<&'a str> }

pub fn load<'a>(text: String) -> Document<'a> {
    let words = text.split_whitespace().collect();
    Document { text, words }
}
