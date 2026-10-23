// A5 (i): returning a reference needs a lifetime when two inputs are references.
pub fn longer(a: &str, b: &str) -> &str {
    if a.len() >= b.len() { a } else { b }
}
