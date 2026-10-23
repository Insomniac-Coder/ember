# A1 again, but `note` is `pub` (a library type): [BRW-10] then borrows all of `self`.
pub struct Player:
    pub name: String
    pub hp: int

pub struct Game:
    pub players: Array[Player]
    pub log: Array[String]
    pub frame: int = 0

    pub fn note(mut self, msg: String):
        self.log.push(f"[{self.frame}] {msg}")

    pub fn tick(mut self):
        for p in self.players.iter_mut():
            p.hp -= 1
            if p.hp <= 0:
                self.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    g.tick()
    println(g.log)
