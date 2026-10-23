# A6: storing view structs in a long-lived collection (a list of handles into one registry).
struct Registry:
    hp: Array[int]

struct Handle:
    reg: ref mut Registry
    entity: int

fn main():
    reg = Registry(hp=[10, 10])
    hs: Array[Handle] = []
    hs.push(Handle(reg=ref mut reg, entity=0))
    hs.push(Handle(reg=ref mut reg, entity=1))
    println(hs.len())
