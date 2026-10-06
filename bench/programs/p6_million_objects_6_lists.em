class Player:
    a: Array[int]
    b: Array[int]
    c: Array[int]
    d: Array[int]
    e: Array[int]
    f: Array[int]
    hp: int

    fn init(mut self, hp: int):
        self.a = []
        self.b = []
        self.c = []
        self.d = []
        self.e = []
        self.f = []
        self.hp = hp

fn main():
    players: Array[Player] = []
    for i in 0..1000000:
        players.push(Player(i % 100))
    total = 0
    for round in 0..40:
        for p in players:
            total = total + p.hp + len(p.f)
    println(total)
