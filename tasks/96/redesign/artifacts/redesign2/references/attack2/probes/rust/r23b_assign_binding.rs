// Twin of y23b: assigning to the whole binding. Rust makes the write-through explicit (`*path = ...`);
// a plain `path = ...` is a type error, never a silent write into the stored state.
pub enum State { Idle, Walking(Vec<i64>) }
pub struct Npc { pub state: State }
impl Npc {
    pub fn peek_or_default(&mut self) -> usize {
        match &mut self.state {
            State::Walking(path) => {
                if path.len() > 3 { path = vec![0]; }
                path.len()
            }
            State::Idle => 0,
        }
    }
}
