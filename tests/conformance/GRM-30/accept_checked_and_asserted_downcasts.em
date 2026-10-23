#$ test: run-pass
#$ rules: GRM-30, TYP-6
#$ stdout: 7
#$ stdout: 7
## `[GRM-30]` — `cast a if Dog` is a checked downcast to `Option[Dog]`,
## `cast a as Dog` one that panics on failure (ODR-280).

open class Animal:
    fn name(self) -> int:
        return 0

class Dog(Animal):
    fn bark(self) -> int:
        return 7

fn main():
    a: Animal = Dog()
    match cast a if Dog:
        Some(dog):
            println(dog.bark())
        None:
            println("no")
    println((cast a as Dog).bark())
