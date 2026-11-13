# Clause BRW-16.6: "a name for an element of any other container (a `Deque`, a `SmallArray`, a
# `Map`'s value, a type of the program's own that implements `Index`) is made by a call of that
# container's `index`, a view a call gives, and is an address (`[BRW-17]`), which every reshape of the
# container ends, adding to it included."
#$ test: compile-fail
#$ rules: BRW-16, BRW-17, STD-11, STD-17, TYP-21
# Adding ends each name here, as it would not end an `Array`'s position. Codes only.

struct Enemy:
    hp: int

struct Bag:
    items: Array[Enemy]

extend Bag implements Index[int]:
    type Output = Enemy

    fn index(i: int) -> ref Enemy:
        return ref this.items[i]

fn a_maps_value():
    m: Map[String, Enemy] = {String("a"): Enemy(hp=1)}
    e = m["a"]
    m.insert(owns String("b"), owns Enemy(hp=2))
    println(e.hp)    #$ error[E3060]

fn an_index_of_the_programs_own():
    bag = Bag(items=[Enemy(hp=1)])
    e = bag[0]
    bag.items.push(owns Enemy(hp=2))
    println(e.hp)    #$ error[E3060]

fn main():
    a_maps_value()
    an_index_of_the_programs_own()
