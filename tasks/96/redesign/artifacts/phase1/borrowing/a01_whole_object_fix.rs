// A1 workaround: make `note` a free function over the one field it needs.
pub struct Player { pub name: String, pub hp: i32 }
pub struct Game { players: Vec<Player>, log: Vec<String>, frame: u64 }

fn note(log: &mut Vec<String>, frame: u64, msg: String) {
    log.push(format!("[{}] {}", frame, msg));
}
impl Game {
    pub fn tick(&mut self) {
        for p in self.players.iter_mut() {
            p.hp -= 1;
            if p.hp <= 0 {
                note(&mut self.log, self.frame, format!("{} died", p.name));
            }
        }
    }
}
