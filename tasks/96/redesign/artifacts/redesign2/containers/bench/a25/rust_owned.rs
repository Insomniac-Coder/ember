// Safe Rust: owned words, Vec<String>, one allocation per word (what a Rust programmer writes when
// the list must outlive the text).
include!("../rharness.rs");
const TEXT_BYTES: usize = 32 << 20;
fn make_text() -> Vec<u8> {
    let mut t = Vec::with_capacity(TEXT_BYTES);
    h::seed(11);
    while t.len() + 14 < TEXT_BYTES {
        let wl = 1 + h::rand() % 12;
        for _ in 0..wl {
            let r = h::rand();
            let mut c = b'a' + (r % 26) as u8;
            if (r >> 8) % 8 == 0 { c = c - b'a' + b'A'; }
            t.push(c);
        }
        t.push(b' ');
    }
    t
}
fn fnv(p: &[u8]) -> u64 {
    let mut h: u64 = 1469598103934665603;
    for &c in p { h ^= c as u64; h = h.wrapping_mul(1099511628211); }
    h
}
fn main() {
    let t0 = h::start();
    let text = String::from_utf8(make_text()).unwrap();
    let mut total: u64 = 0;
    for _ in 0..5 {
        let low = text.to_ascii_lowercase();
        let words: Vec<String> = low.split_ascii_whitespace().map(|w| w.to_string()).collect();
        let mut x: u64 = 0;
        for w in &words { x ^= fnv(w.as_bytes()); }
        total = total.wrapping_add(x.wrapping_add(words.len() as u64));
    }
    h::report("a25-rust-owned", t0, total);
}
