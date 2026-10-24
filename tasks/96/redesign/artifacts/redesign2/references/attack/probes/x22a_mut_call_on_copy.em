# Attack on A22's "a temporary is a place for its statement": a `mut self` method on a COPY.
# Python's reg.get(0).heal(100) changes the stored hero; here `get` returns a copy, so the heal is lost
# with no message. Today this is E2140, which is the only thing that catches it.
@derive(Copy)
struct Hero:
    hp: int

    fn heal(mut self, n: int) -> bool:
        self.hp += n
        return self.hp > 50

struct Registry:
    heroes: Array[Hero]

    fn get(self, i: int) -> Hero:
        return self.heroes[i]

fn main():
    reg = Registry(heroes=[Hero(hp=10)])
    if reg.get(0).heal(100):
        println("healed")
    println(reg.heroes[0].hp)
