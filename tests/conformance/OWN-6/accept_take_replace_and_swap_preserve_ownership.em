#$ test: run-pass
#$ rules: OWN-6, OWN-3, DRP-2
#$ stdout:
#$ 1
#$ 1
#$ 2
#$ 0
#$ 2
#$ 3
#$ 0
#$ 0
#$ 3

import std.mem
from std.core import Default

struct Item:
    pub value: i32

    fn drop(mut self):
        println(self.value)

extend Item implements Default:
    fn default() -> Item:
        return Item(0)

fn main():
    current = Item(1)
    replaced = mem.replace(current, Item(2))
    println(replaced.value)
    mem.drop(replaced)

    taken = mem.take(current)
    println(taken.value)
    println(current.value)
    mem.drop(taken)

    other = Item(3)
    mem.swap(current, other)
    println(current.value)
    println(other.value)
