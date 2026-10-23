// A12: a button handler that bumps the score.
pub struct Button { handlers: Vec<Box<dyn FnMut()>> }
impl Button {
    pub fn on_click(&mut self, f: impl FnMut() + 'static) { self.handlers.push(Box::new(f)); }
    pub fn click(&mut self) { for h in self.handlers.iter_mut() { h() } }
}

pub fn game() -> i32 {
    let mut score = 0;
    let mut b = Button { handlers: Vec::new() };
    b.on_click(|| score += 1);
    b.click();
    score
}

// Adding `move` compiles, but the closure bumps its own copy: this returns 0.
pub fn game_move() -> i32 {
    let mut score = 0;
    let mut b = Button { handlers: Vec::new() };
    b.on_click(move || score += 1);
    b.click();
    score
}
