// A1: a method borrows all of `self` while a loop holds one field.
pub struct Player { pub name: String, pub hp: i32 }
pub struct Game { players: Vec<Player>, log: Vec<String>, frame: u64 }

impl Game {
    fn note(&mut self, msg: String) {
        self.log.push(format!("[{}] {}", self.frame, msg));
    }
    pub fn tick(&mut self) {
        for p in self.players.iter_mut() {
            p.hp -= 1;
            if p.hp <= 0 {
                self.note(format!("{} died", p.name));
            }
        }
    }
}
