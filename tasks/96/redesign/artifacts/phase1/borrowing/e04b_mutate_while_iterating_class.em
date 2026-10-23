# A4 on a class's field: a loop over `self.enemies` that pushes onto it.
class Enemy:
    hp: int
    splits: bool

class World:
    enemies: Array[Enemy]

    fn update(self):
        for e in self.enemies:
            if e.hp <= 0 and e.splits:
                self.enemies.push(Enemy(hp=5, splits=false))

fn main():
    w = World(enemies=[Enemy(hp=0, splits=true)])
    w.update()
    println(w.enemies.len())
