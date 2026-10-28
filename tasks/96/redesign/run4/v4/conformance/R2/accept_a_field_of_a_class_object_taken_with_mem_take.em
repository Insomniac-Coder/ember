#$ test: run-pass
#$ rules: R2
#$ stdout: ann 0
## R2: the nearest program that is allowed: `mem.take(ref mut q.name)` takes the name out of the class
## object and leaves an empty `String` in its place (verdict 78).

import std.mem

class Player:
    name: String

fn main():
    q = Player(name=String("ann"))
    names: Array[String] = []
    names.push(mem.take(ref mut q.name))
    println(names[0], q.name.len())
