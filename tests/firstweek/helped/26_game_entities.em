# Game entities with an open base class: goblins, a troll and a hero override how they act in a small turn-based battle.

open class Entity:
    let name: String
    pub(read) health: int
    attack: int

    fn init(self, owned name: String, health: int, attack: int):
        self.name = name
        self.health = health
        self.attack = attack

    fn alive(self) -> bool:
        return self.health > 0

    fn take_damage(self, amount: int):
        self.health = max(self.health - amount, 0)
        println(f"    {self.name} takes {amount} damage ({self.health} left)")

    virtual fn act(self, target: Entity):
        target.take_damage(self.attack)

    virtual fn describe(self) -> String:
        return f"{self.name} ({self.health} hp)"

class Goblin(Entity):
    override fn act(self, target: Entity):
        println(f"  {self.name} stabs wildly")
        target.take_damage(self.attack)

class Troll(Entity):
    resting: bool = false

    override fn act(self, target: Entity):
        if self.resting:
            println(f"  {self.name} is catching its breath")
            self.resting = false
            return
        println(f"  {self.name} swings a club")
        target.take_damage(self.attack * 2)
        self.resting = true

class Hero(Entity):
    potions: int = 2

    override fn act(self, target: Entity):
        if self.health < 15 and self.potions > 0:
            self.potions -= 1
            self.health += 20
            println(f"  {self.name} drinks a potion ({self.potions} left)")
        else:
            println(f"  {self.name} strikes {target.name}")
            target.take_damage(self.attack)

    override fn describe(self) -> String:
        return f"{self.name} the hero ({self.health} hp, {self.potions} potions)"

fn main():
    hero = Hero("Ayla", 40, 7)
    monsters: Array[Entity] = [Goblin("Snik", 12, 3), Troll("Grum", 30, 4), Goblin("Nab", 10, 3)]
    turn = 1
    while hero.alive() and any(m.alive() for m in monsters):
        println(f"turn {turn}: {hero.describe()}")
        target = [m for m in monsters if m.alive()][0]
        hero.act(target)
        for m in monsters:
            if m.alive() and hero.alive():
                m.act(hero)
        turn += 1
        if turn > 30:
            break

    if hero.alive():
        println(f"{hero.name} wins after {turn - 1} turns")
    else:
        println(f"{hero.name} has fallen")
    for m in monsters:
        println(f"  {m.describe()}")
