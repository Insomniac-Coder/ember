// A23: replacing a field's value computed from its old value, through &mut self.
pub enum State { Idle, Walking(Vec<(f32, f32)>), Dead }
pub struct Npc { pub state: State }

impl Npc {
    pub fn update(&mut self) {
        self.state = match self.state {
            State::Walking(mut path) => {
                path.pop();
                if path.is_empty() { State::Idle } else { State::Walking(path) }
            }
            other => other,
        };
    }
}
