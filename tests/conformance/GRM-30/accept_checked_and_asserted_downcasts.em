#$ test: run-pass
#$ rules: GRM-30, TYP-6
#$ stdout: 7
#$ stdout: 7
## `[GRM-30]` — `as?` and `as!` are single tokens: `a as? Dog` is a checked
## downcast to `Option[Dog]`, `a as! Dog` one that panics on failure.

open class Animal:
    fn name(self) -> int:
        return 0

class Dog(Animal):
    fn bark(self) -> int:
        return 7

fn main():
    a: Animal = Dog()
    match a as? Dog:
        Some(dog):
            println(dog.bark())
        None:
            println("no")
    println((a as! Dog).bark())
