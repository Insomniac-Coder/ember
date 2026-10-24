// Twin of p06a: read-only handles kept in a local Vec.
pub struct Registry { pub hp: Vec<i32> }
pub struct View<'a> { pub reg: &'a Registry, pub entity: usize }
pub fn total(reg: &Registry) -> i32 {
    let mut vs: Vec<View> = Vec::new();
    vs.push(View { reg, entity: 0 });
    vs.push(View { reg, entity: 1 });
    vs.iter().map(|v| v.reg.hp[v.entity]).sum()
}
