# Lobster/Nim/Koka style: elements are counted objects; a held element is a handle, not a pointer into the buffer
class Player:
    pub hp: int

    fn init(mut self, hp: int):
        self.hp = hp

fn main():
    players: Array[Player] = []
    players.push(Player(10))
    p = players[0]
    players.push(Player(20))
    p.hp -= 1
    println(players[0].hp)
