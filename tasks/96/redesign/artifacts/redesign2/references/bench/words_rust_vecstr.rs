// Safe Rust, the form Rust accepts: Vec<&str> borrowing a String owned outside it (no self-reference).
include!("harness.rs");
include!("words_common.rs");
fn main() {
    let mut h = H::start(); h.seed(42);
    let text = make_text(&mut h);
    let t0 = std::time::Instant::now();
    let words: Vec<&str> = text.split(' ').collect();
    let mut sum = 0u64;
    for _ in 0..PASSES { for w in &words { sum = sum.wrapping_add(fnv(w.as_bytes())); } }
    for _ in 0..NRAND {
        let j = (h.rand() as usize) % words.len();
        let w = words[j];
        sum = sum.wrapping_add(w.len() as u64 + w.as_bytes()[0] as u64);
    }
    drop(words); drop(text);
    print!("work_ms={:.3} ", t0.elapsed().as_secs_f64() * 1000.0);
    h.report("words-rust-vecstr", sum);
}
