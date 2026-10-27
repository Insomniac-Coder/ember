// rust_cb_parse.rs: workload cb_parse in Rust: the C library (written here in Rust, `extern "C"`,
// not inlined, the callbacks hidden by `black_box`) calls back through raw user-data pointers turned
// into `&mut` with `unsafe`, the workaround for C callbacks with state; nothing is checked.
// Same text, same work as plain_cb_parse.c.
// Build: rustc -C opt-level=2 --edition 2021 rust_cb_parse.rs
mod harness;
use std::os::raw::c_void;

const FIELDS: usize = 2_000_000;
const PER_ROW: usize = 8;
const PASSES: usize = 10;

type FieldCb = extern "C" fn(*mut c_void, i64);
type RowCb = extern "C" fn(*mut c_void);

#[inline(never)]
extern "C" fn csv_parse(text: *const u8, n: usize, on_field: FieldCb, field_ud: *mut c_void, on_row: RowCb, row_ud: *mut c_void) {
    let text = unsafe { std::slice::from_raw_parts(text, n) };   // C's view of the buffer
    let mut v: i64 = 0;
    for &c in text {
        if c.is_ascii_digit() { v = v * 10 + (c - b'0') as i64; }
        else if c == b',' { on_field(field_ud, v); v = 0; }
        else if c == b'\n' { on_field(field_ud, v); v = 0; on_row(row_ud); }
    }
}

pub struct Stats { pub sum: i64, pub count: i64, pub max: i64 }

extern "C" fn on_field(ud: *mut c_void, v: i64) {
    let s: &mut Stats = unsafe { &mut *(ud as *mut Stats) };
    s.sum += v; s.count += 1;
    if v > s.max { s.max = v; }
}
extern "C" fn on_row(ud: *mut c_void) {
    let rows: &mut i64 = unsafe { &mut *(ud as *mut i64) };
    *rows += 1;
}

fn main() {
    let t0 = harness::start();
    let mut rng = harness::Rng::new(41);
    let mut text: Vec<u8> = Vec::with_capacity(FIELDS * 7);
    for i in 0..FIELDS {
        let mut v = rng.next() % 100000;
        let mut digits = [0u8; 8]; let mut nd = 0;
        loop { digits[nd] = b'0' + (v % 10) as u8; nd += 1; v /= 10; if v == 0 { break; } }
        while nd > 0 { nd -= 1; text.push(digits[nd]); }
        text.push(if i % PER_ROW == PER_ROW - 1 { b'\n' } else { b',' });
    }
    let mut st = Stats { sum: 0, count: 0, max: 0 };
    let mut rows: i64 = 0;
    let f = std::hint::black_box(on_field as FieldCb);
    let r = std::hint::black_box(on_row as RowCb);
    for _ in 0..PASSES {
        csv_parse(text.as_ptr(), text.len(), f, &mut st as *mut Stats as *mut c_void, r, &mut rows as *mut i64 as *mut c_void);
    }
    let sum = (st.sum as u64).wrapping_add((st.count as u64).wrapping_mul(7)).wrapping_add((st.max as u64).wrapping_mul(13)).wrapping_add((rows as u64).wrapping_mul(17));
    harness::report("rust_cb_parse", t0, sum);
}
