// Twin of y23a: the Python-shaped state machine in a loop over the elements. Rust refuses the move.
pub enum State { Idle, Walking(Vec<i64>), Running(Vec<i64>) }
pub struct Npc { pub state: State }
pub fn tick(npcs: &mut Vec<Npc>, hurry: bool) {
    for npc in npcs.iter_mut() {
        match &mut npc.state {
            State::Walking(path) => {
                path.pop();
                if path.is_empty() { npc.state = State::Idle; }
                else if hurry { npc.state = State::Running(*path); }
            }
            _ => {}
        }
    }
}
