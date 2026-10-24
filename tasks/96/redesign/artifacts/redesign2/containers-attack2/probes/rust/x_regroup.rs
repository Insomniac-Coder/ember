// Rust twin of demo C: one squad passed as both parameters.
pub struct Unit { pub name: String, pub hp: i32 }
pub struct Squad { pub units: Vec<Unit> }
pub fn regroup(a: &mut Squad, b: &Squad) -> usize { let first_two = &b.units[0..2]; a.units.clear(); first_two[0].name.len() }
pub fn caller(s: &mut Squad) -> usize { regroup(s, s) }
