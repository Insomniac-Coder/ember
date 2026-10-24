# A2 attack 2: an element passed to a method that removes other elements of its list. Rule 6 passes
# the element by copy and writes it back to its old position, which now holds another enemy.
struct Enemy:
    hp: int

class World:
    enemies: Array[Enemy]

    fn init(mut self):
        self.enemies = [Enemy(hp=0), Enemy(hp=50), Enemy(hp=70)]

    fn purge(mut self):
        self.enemies.retain(fn(e: Enemy) -> bool: e.hp > 0)

    fn hit(mut self, mut e: Enemy):
        e.hp -= 10
        self.purge()

fn main():
    w = World()
    w.hit(w.enemies[1])
    println(w.enemies.len())
