// A23 workaround: std::mem::replace with a placeholder value.
pub enum State { Idle, Walking(Vec<(f32, f32)>), Dead }
pub struct Npc { pub state: State }

impl Npc {
    pub fn update(&mut self) {
        let old = std::mem::replace(&mut self.state, State::Idle);
        self.state = match old {
            State::Walking(mut path) => {
                path.pop();
                if path.is_empty() { State::Idle } else { State::Walking(path) }
            }
            other => other,
        };
    }
}
