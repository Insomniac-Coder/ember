// Safe Rust twins of the line loop (lines.h). Build with --cfg twophase for the Rust workaround that
// keeps views (collect the lowered lines first, then a Vec<&str> into them: the same memory as the
// take-along answer), or without it for owned words (Vec<String>, one allocation per word).
include!("../rharness.rs");
const TEXT_BYTES: usize = 32 << 20;
fn make_lines() -> Vec<u8> {
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
    let mut col = 0usize;
    for c in t.iter_mut() {
        if *c == b' ' && col >= 72 { *c = b'\n'; col = 0; } else { col += 1; }
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
    let text = String::from_utf8(make_lines()).unwrap();
    let mut total: u64 = 0;
    for _ in 0..5 {
        #[cfg(twophase)]
        let (x, n) = {
            let lines: Vec<String> = text.split('\n').map(|l| l.to_ascii_lowercase()).collect();
            let words: Vec<&str> = lines.iter().flat_map(|l| l.split(' ').filter(|w| !w.is_empty())).collect();
            let mut x: u64 = 0;
            for w in &words { x ^= fnv(w.as_bytes()); }
            (x, words.len())
        };
        #[cfg(not(twophase))]
        let (x, n) = {
            let mut words: Vec<String> = Vec::new();
            for line in text.split('\n') {
                let low = line.to_ascii_lowercase();
                for w in low.split(' ').filter(|w| !w.is_empty()) { words.push(w.to_string()); }
            }
            let mut x: u64 = 0;
            for w in &words { x ^= fnv(w.as_bytes()); }
            (x, words.len())
        };
        total = total.wrapping_add(x.wrapping_add(n as u64));
    }
    #[cfg(twophase)]
    h::report("a25-lines-rust-twophase", t0, total);
    #[cfg(not(twophase))]
    h::report("a25-lines-rust-owned", t0, total);
}
