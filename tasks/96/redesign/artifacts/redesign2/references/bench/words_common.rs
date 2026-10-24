// words_common.rs: the Rust side of words_common.h (same text, same hash, same work).
const NW: u32 = 4_000_000;
const PASSES: u32 = 10;
const NRAND: u32 = 8_000_000;
fn make_text(h: &mut H) -> String {
    let mut t: Vec<u8> = Vec::with_capacity(NW as usize * 13);
    for i in 0..NW {
        if i != 0 { t.push(b' '); }
        let n = 1 + h.rand() % 12;
        for _ in 0..n { t.push(b'a' + (h.rand() % 26) as u8); }
    }
    String::from_utf8(t).unwrap()
}
#[inline]
fn fnv(b: &[u8]) -> u64 {
    let mut h = 1469598103934665603u64;
    for &x in b { h ^= x as u64; h = h.wrapping_mul(1099511628211); }
    h
}
