// A3: a view of a.name kept across a write to b.name; two &mut never alias, so Rust accepts it.
pub struct Unit { pub name: String }
pub fn rename_both(a: &mut Unit, b: &mut Unit) -> usize {
    let old: &str = &a.name;
    b.name = String::from("renamed");
    old.len()
}
