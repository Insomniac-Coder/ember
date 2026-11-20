# Clause EFF-27.1: "a link to the whole holder is allowed, since every change through it passes
# through the holder"; "the kept result is cleared by every change, through any name, to one of
# them: the compiler places the clear at each line that may make such a change"
#$ test: run-pass
#$ rules: EFF-27, CLS-12, LT-46
#$ stdout: a
#$ stdout: b
#$ stdout: c
# The message's way out (`sharedref mut s`): a change through the link clears the kept name, and so
# does one made by a function given the shelf `ref mut`.

struct Book:
    name: String

class Shelf:
    items: Array[Book] = []

    @cached
    fn first_name() -> String:
        return copy of this.items[0].name

class Watcher:
    target: sharedref mut Shelf

fn rename_first(ref mut sh: Shelf):
    sh.items[0].name = String("c")

fn main():
    s = Shelf()
    s.items.push(owns Book(String("a")))
    println(s.first_name())
    w = Watcher(target=sharedref mut s)
    w.target.items[0].name = String("b")
    println(s.first_name())
    rename_first(ref mut s)
    println(s.first_name())
