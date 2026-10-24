# A1 control: the same `pub` method and loop as a1_two_packages, but in one package.
pub struct Player:
    pub name: String
    pub hp: int

pub struct Game:
    pub players: Array[Player]
    pub log: Array[String]
    pub frame: int = 0

    pub fn note(mut self, msg: String):
        self.log.push(f"[{self.frame}] {msg}")

fn tick(mut g: Game):
    for p in g.players.iter_mut():
        p.hp -= 1
        if p.hp <= 0:
            g.note(f"{p.name} died")

fn main():
    g = Game(players=[Player(name="orc", hp=1)], log=[])
    tick(g)
    println(g.log)
