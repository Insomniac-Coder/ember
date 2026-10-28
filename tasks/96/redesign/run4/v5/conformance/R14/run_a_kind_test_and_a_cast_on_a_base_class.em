#$ test: run-pass
#$ rules: TYP-44
#$ stdout: 7
#$ stdout: 3
#$ stdout: 7
## design rules: R14
## R14: "A value of an interface type (`dyn I`, `owns dyn I`, `shares dyn I`) or of a base class is
## tested for its kind with `is`"; "`cast s as Circle` gives the `Circle`"; "A cast of a place is a
## place: the name bound from it is read-only".

open class Animal:
    legs: int = 4

class Dog(Animal):
    fn bark() -> int:
        return 7

class Cat(Animal):
    fn purr() -> int:
        return 3

fn sound(a: Animal) -> int:
    if a is Dog as d:
        return d.bark()
    elif a is Cat as c:
        return c.purr()
    return 0

fn main():
    pets: Array[Animal] = [Dog(), Cat()]
    for p in pets:
        println(sound(p))
    first = cast pets[0] as Dog
    println(first.bark())
