# A4 through a method, closest form today: iter_mut, then self.kill(i) removes the walked element.
struct Enemy:
    hp: int

struct World:
    enemies: Array[Enemy]
    score: int

    fn kill(mut self, i: int):
        self.score += 10
        _gone = self.enemies.remove(i)

    fn update(mut self):
        for i, e in enumerate(self.enemies.iter_mut()):
            e.hp -= 1
            if e.hp <= 0:
                self.kill(i)

fn main():
    w = World(enemies=[Enemy(hp=1), Enemy(hp=3)], score=0)
    w.update()
    println(w.enemies.len())
