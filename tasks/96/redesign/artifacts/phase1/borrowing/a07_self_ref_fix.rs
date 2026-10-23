// A7 workaround: keep ranges (indices) instead of slices.
pub struct Document { pub text: String, pub words: Vec<(usize, usize)> }

pub fn load(text: String) -> Document {
    let mut words = Vec::new();
    let mut start = None;
    for (i, c) in text.char_indices() {
        match (c.is_whitespace(), start) {
            (false, None) => start = Some(i),
            (true, Some(s)) => { words.push((s, i)); start = None; }
            _ => {}
        }
    }
    if let Some(s) = start { words.push((s, text.len())); }
    Document { text, words }
}
impl Document {
    pub fn word(&self, k: usize) -> &str { let (a, b) = self.words[k]; &self.text[a..b] }
}
