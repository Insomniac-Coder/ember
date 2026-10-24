// Rust twin of demo R: a slice of squads[0].units passed with &mut squads.
pub struct Squad { pub units: Vec<i32> }
pub fn reinforce(squads: &mut [Squad], picked: &[i32]) { for s in squads.iter_mut() { s.units.extend_from_slice(picked); } }
pub fn caller(squads: &mut Vec<Squad>) { reinforce(squads, &squads[0].units[0..2]); }
