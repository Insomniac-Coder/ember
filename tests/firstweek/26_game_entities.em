# Game entities with an open base class: goblins, a troll and a hero override how they act in a small turn-based battle.

open class Entity:
    let name: String
    pub(read) health: int
    attack: int

    fn init(owns name: String, health: int, attack: int):
        this.name = name
        this.health = health
        this.attack = attack

    fn alive() -> bool:
        return this.health > 0

    fn take_damage(amount: int):
        this.health = max(this.health - amount, 0)
        println(f"    {this.name} takes {amount} damage ({this.health} left)")

    virtual fn act(target: Entity):
        target.take_damage(this.attack)

    virtual fn describe() -> String:
        return f"{this.name} ({this.health} hp)"

class Goblin(Entity):
    override fn act(target: Entity):
        println(f"  {this.name} stabs wildly")
        target.take_damage(this.attack)

class Troll(Entity):
    resting: bool = False

    override fn act(target: Entity):
        if this.resting:
            println(f"  {this.name} is catching its breath")
            this.resting = False
            return
        println(f"  {this.name} swings a club")
        target.take_damage(this.attack * 2)
        this.resting = True

class Hero(Entity):
    potions: int = 2

    override fn act(target: Entity):
        if this.health < 15 and this.potions > 0:
            this.potions -= 1
            this.health += 20
            println(f"  {this.name} drinks a potion ({this.potions} left)")
        else:
            println(f"  {this.name} strikes {target.name}")
            target.take_damage(this.attack)

    override fn describe() -> String:
        return f"{this.name} the hero ({this.health} hp, {this.potions} potions)"

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
