// Twin of p05f: a conditionally chosen mutable part.
pub fn smaller(a: &mut i32, b: &mut i32) -> &mut i32 {
    if *a <= *b { a } else { b }
}
