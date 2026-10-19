#$ test: run-pass
#$ rules: OWN-3, DRP-2
#$ profiles: debug, development, release
#$ stdout: drop b
#$ end
#$ drop a
# `[OWN-3]` — replacing a field of a class instance drops the field's old value and leaves the handle
# as it was: `a` still names its object, which its scope's end drops. (D-698: dropping the old value
# of `a.spare` was taken to move `a`, which the write put back; once a write no longer counts as
# giving `a` a value, `a` would otherwise never have been dropped.)

class Leaf:
    name: String

    fn drop(mut self):
        println(f"drop {self.name}")

class Node:
    own: Leaf
    spare: Option[Leaf] = None

fn main():
    a = Node(Leaf("a"))
    a.spare = Some(Leaf("b"))
    a.spare = None
    println("end")
