#$ test: run-pass
#$ rules: DSP-2, DSP-1
#$ stdout: base 1
#$ stdout: derived 2
#$ stdout: base 1
# `[DSP-2]` — a virtual call loads its slot from the object's type table; an `override` reuses the
# base's slot, and a slot a subclass does not override keeps the base's entry.

open class Animal:
    virtual fn name() -> String:
        return String("base")

    virtual fn legs() -> int:
        return 1

class Dog(Animal):
    override fn name() -> String:
        return String("derived")

    override fn legs() -> int:
        return 2

class Fish(Animal):
    pass

fn show(a: Animal):
    println(a.name(), a.legs())

fn main():
    base = Animal()
    dog = Dog()
    fish = Fish()
    show(base)
    show(dog)
    show(fish)
