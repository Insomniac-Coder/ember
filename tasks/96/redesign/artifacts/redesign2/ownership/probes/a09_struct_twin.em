# A9 twin with struct values: one Player owned by a Game, borrowed by two loops' worth of code.
struct Player:
    hp: int
    inventory: Array[String]
    log: Array[String]

fn main():
    p = Player(hp=100, inventory=["key"], log=[])
    for item in p.inventory:
        p.hp -= item.len()          # a Copy field: disjoint from the loop's field ([BRW-4])
    for item in p.inventory:
        p.log.push(item.clone())    # another non-Copy field: disjoint
    for item in p.inventory:
        p.inventory.push(item.clone())  # the same field: refused (E3020)
