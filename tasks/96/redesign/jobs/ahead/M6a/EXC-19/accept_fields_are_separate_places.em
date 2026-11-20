# Clause EXC-19.1: "A view of one field of an object and a change of another field never conflict:
# `for c in this.children: this.log.push(owns copy of c.name)` is accepted. What a method's call
# touches is the fields its body touches (`[BRW-10]`)."
#$ test: run-pass
#$ rules: EXC-19, BRW-10, BRW-18
#$ stdout: 2
#$ stdout: a
#$ stdout: b
#$ stdout: 2
# The spec's loop, and the same with the push made by a method whose body touches only `log`.

class Child:
    name: String

    fn init(owns name: String):
        this.name = name

class Parent:
    children: Array[Child] = []
    log: Array[String] = []

    fn note(owns s: String):
        this.log.push(owns s)

    fn record():
        for c in this.children:
            this.log.push(owns copy of c.name)

    fn record_by_method():
        for c in this.children:
            this.note(owns copy of c.name)

fn main():
    p = Parent()
    p.children.push(owns Child(String("a")))
    p.children.push(owns Child(String("b")))
    p.record()
    println(p.log.len())
    for line in p.log:
        println(line)
    p.log.clear()
    p.record_by_method()
    println(p.log.len())
