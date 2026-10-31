class Player:
    a: Array[int]
    b: Array[int]
    c: Array[int]
    d: Array[int]
    e: Array[int]
    f: Array[int]
    hp: int

    fn init(hp: int):
        this.a = []
        this.b = []
        this.c = []
        this.d = []
        this.e = []
        this.f = []
        this.hp = hp

fn main():
    players: Array[Player] = []
    for i in 0..1000000:
        players.push(Player(i % 100))
    total = 0
    for round in 0..40:
        for p in players:
            total = total + p.hp + len(p.f)
    println(total)
