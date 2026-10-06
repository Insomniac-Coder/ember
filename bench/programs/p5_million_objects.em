class Player:
    name: String
    items: Array[int]
    hp: int

    fn init(mut self, hp: int):
        self.name = ""
        self.items = []
        self.hp = hp

fn main():
    players: Array[Player] = []
    for i in 0..1000000:
        players.push(Player(i % 100))
    total = 0
    for round in 0..40:
        for p in players:
            total = total + p.hp + len(p.items)
    println(total)
