# A4 with counted elements: spawning while iterating
class Enemy:
    pub hp: int

    fn init(mut self, hp: int):
        self.hp = hp

fn main():
    enemies: Array[Enemy] = []
    enemies.push(Enemy(10))
    for e in enemies:
        e.hp -= 1
        enemies.push(Enemy(5))
    println(enemies.len())
