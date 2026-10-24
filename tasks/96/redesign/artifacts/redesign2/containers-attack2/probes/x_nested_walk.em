# A4 attack: a loop walks a list inside a list held by a class; code the compiler cannot see (a stored
# function value) replaces the outer element while the loop runs.
struct Unit:
    hp: int

struct Squad:
    units: Array[Unit]

class World:
    squads: Array[Squad]
    on_step: fn(World)

    fn init(mut self, owned on_step: fn(World)):
        self.squads = [Squad(units=[Unit(hp=1), Unit(hp=0), Unit(hp=3)])]
        self.on_step = on_step

    fn update(mut self):
        for u in self.squads[0].units:
            u.hp -= 1
            if u.hp <= 0:
                _gone = self.squads[0].units.remove(0)   # the answer's spelling: remove(u)
            self.on_step(self)

fn replace_first(w: World):
    w.squads[0] = Squad(units=[Unit(hp=7)])

fn main():
    w = World(owned fn(x: World) => replace_first(x))
    w.update()
    println(w.squads[0].units.len())
