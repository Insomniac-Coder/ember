// A3: extending a Vec with a slice of itself.
pub fn grow(v: &mut Vec<i32>) {
    v.extend_from_slice(&v[0..2]);
}
