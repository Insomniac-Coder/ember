# W3 in the combined model's proposed syntax (phase3b-combined.md 6.3). Checked with today's
# `ember check` only to record what today's compiler says.
state: u64 = 7

fn rand() -> int:
    global state
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)
    return (state >> 33) as int

open class GameObject:
    id: int
    x: i32
    y: i32
    health: i32
    target: weak GameObject = None

    virtual fn update(self):
        pass

class Ship(GameObject):
    override fn update(self):
        t = self.target
        if t is not None:
            self.x += sign(t.x - self.x)
            self.y += sign(t.y - self.y)

class Rock(GameObject):
    override fn update(self):
        self.x = (self.x + 1) % 1000

class Turret(GameObject):
    override fn update(self):
        t = self.target
        if t is not None and (t.x - self.x).abs() + (t.y - self.y).abs() < 300:
            t.health -= 1

fn sign(d: i32) -> i32:
    if d > 0:
        return 1
    if d < 0:
        return -1
    return 0

fn make(id: int) -> GameObject:
    x = (rand() % 1000) as i32
    y = (rand() % 1000) as i32
    health = (100 + id % 50) as i32
    if id % 3 == 0:
        return Ship(id=id, x=x, y=y, health=health)
    if id % 3 == 1:
        return Rock(id=id, x=x, y=y, health=health)
    return Turret(id=id, x=x, y=y, health=health)

fn main():
    objects: Array[GameObject] = []
    for i in 0..100_000:
        o = make(i)
        if i > 0:
            o.target = objects[rand() % i]
        objects.push(o)
    next_id = 100_000
    destroyed = 0
    for frame in 0..1000:
        for o in objects:
            o.update()
        dead = 0
        for o in objects:
            if o.health <= 0:
                objects.remove(o)
                dead += 1
        destroyed += dead
        for _ in 0..dead:
            o = make(next_id)
            next_id += 1
            if len(objects) > 0:
                o.target = objects[rand() % len(objects)]
            objects.push(o)
    total = destroyed * 1_000_000
    for o in objects:
        total += (o.x + o.y + o.health) as int
    println(total)
