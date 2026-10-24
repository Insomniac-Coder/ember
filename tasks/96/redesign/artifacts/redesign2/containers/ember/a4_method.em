# A4 through a method: the game loop calls self.kill(e), which removes e from the list being walked.
struct Enemy:
    hp: int

struct World:
    enemies: Array[Enemy]
    score: int

    fn kill(mut self, e: Enemy):
        self.score += 10
        i = self.enemies.index_of(e)
        match i:
            Some(k):
                _x = self.enemies.remove(k)
            None:
                pass

    fn update(mut self):
        for e in self.enemies:
            e.hp -= 1
            if e.hp <= 0:
                self.kill(e)

fn main():
    w = World(enemies=[Enemy(hp=1), Enemy(hp=3)], score=0)
    w.update()
    println(w.enemies.len())
