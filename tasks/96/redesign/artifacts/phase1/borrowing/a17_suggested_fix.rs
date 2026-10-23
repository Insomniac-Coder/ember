// A17: applying rustc's own `help` from a23_move_out_of_field.rs (borrow the pattern bindings).
pub enum State { Idle, Walking(Vec<(f32, f32)>), Dead }
pub struct Npc { pub state: State }

impl Npc {
    pub fn update(&mut self) {
        self.state = match self.state {
            State::Walking(ref mut path) => {
                path.pop();
                if path.is_empty() { State::Idle } else { State::Walking(path) }
            }
            ref other => other,
        };
    }
}
