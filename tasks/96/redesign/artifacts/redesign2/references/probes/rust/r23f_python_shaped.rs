// Twin of p23f: pop in place, and on one path carry the path into the next state.
pub enum State { Idle, Walking(Vec<i32>), Running(Vec<i32>) }
pub struct Npc { pub state: State }
impl Npc {
    pub fn update(&mut self, hurry: bool) {
        match &mut self.state {
            State::Walking(path) => {
                path.pop();
                if path.is_empty() { self.state = State::Idle; }
                else if hurry { self.state = State::Running(*path); }
            }
            _ => {}
        }
    }
}
