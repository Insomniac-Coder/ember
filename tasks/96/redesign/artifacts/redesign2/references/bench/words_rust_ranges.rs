// Safe Rust, its self-reference workaround (phase 1 a07_self_ref_fix.rs): the document owns its text
// and keeps (start, end) ranges; every use slices the text again (bounds and char-boundary checks).
include!("harness.rs");
include!("words_common.rs");
struct Document { text: String, words: Vec<(usize, usize)> }
impl Document {
    fn load(text: String) -> Document {
        let mut words = Vec::new();
        let mut s = 0;
        for (i, _) in text.match_indices(' ') { words.push((s, i)); s = i + 1; }
        words.push((s, text.len()));
        Document { text, words }
    }
    #[inline]
    fn word(&self, k: usize) -> &str { let (a, b) = self.words[k]; &self.text[a..b] }
}
fn main() {
    let mut h = H::start(); h.seed(42);
    let text = make_text(&mut h);
    let t0 = std::time::Instant::now();
    let doc = Document::load(text);
    let mut sum = 0u64;
    for _ in 0..PASSES { for &(a, b) in &doc.words { sum = sum.wrapping_add(fnv(doc.text[a..b].as_bytes())); } }
    for _ in 0..NRAND {
        let j = (h.rand() as usize) % doc.words.len();
        let w = doc.word(j);
        sum = sum.wrapping_add(w.len() as u64 + w.as_bytes()[0] as u64);
    }
    drop(doc);
    print!("work_ms={:.3} ", t0.elapsed().as_secs_f64() * 1000.0);
    h.report("words-rust-ranges", sum);
}
