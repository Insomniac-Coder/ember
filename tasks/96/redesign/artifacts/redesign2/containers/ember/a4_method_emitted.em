# A4 through a method: what the answer compiles World.update into, written by hand. kill(e) removes
# the element it is given; called on the loop's current element, the compiler uses a copy of kill in
# which that removal becomes "do not keep the current element" (kill_current below). Today's rules
# accept this program.
struct Enemy:
    hp: int

struct World:
    enemies: Array[Enemy]
    score: int

    fn kill_current(mut self) -> bool:     # kill(e) with self.enemies.remove(e) rewritten
        self.score += 10
        return false                       # the current element is not kept

    fn update(mut self):
        r = 0
        w = 0
        while r < self.enemies.len():
            self.enemies[r].hp -= 1
            keep = true
            if self.enemies[r].hp <= 0:
                keep = self.kill_current()
            if keep:
                if w != r:
                    self.enemies.swap(w, r)
                w += 1
            r += 1
        self.enemies.truncate(w)

fn main():
    w = World(enemies=[Enemy(hp=1), Enemy(hp=1), Enemy(hp=5)], score=0)
    w.update()
    println(w.enemies.len())
