# Ember/Rust style: elements stored inline; a held element is a pointer into the buffer
struct Player:
    hp: int

fn main():
    players: Array[Player] = []
    players.push(Player(hp=10))
    p = ref mut players[0]
    players.push(Player(hp=20))
    p.hp -= 1
    println(players[0].hp)
