#$ test: run-pass
#$ rules: RFL-2, RT-3
#$ stdout: Point 2 y 8
#$ stdout: Pair[i64] 2
#$ stdout: Shape 2
#$ stdout: Animal 1 Dog 2 Dog
#$ stdout: true
## `[RFL-2]`, `[RT-3]` — `@reflect` keeps a type's description in the program: `type_info[T]()`
## gives a `ref TypeInfo` with the descriptors `reflect[T]()` gives while compiling, a generic
## type's for each instance, and `h.type_info()` on a class handle the description of the
## object's own class, which its type information holds; a class derived from a `@reflect` class is
## `@reflect` too.

from std.ser import type_info, TypeInfo, Reflect

@reflect
struct Point:
    x: f64
    y: f64

@reflect
struct Pair[T]:
    first: T
    second: T

@reflect
enum Shape:
    Dot
    Circle(f64)

@reflect
open class Animal:
    legs: int

class Dog(Animal):
    tricks: int = 0

fn describe[T: Reflect]() -> int:
    return len(type_info[T]().fields)

fn main():
    p = type_info[Point]()
    println(p.name, len(p.fields), p.fields[1].name, p.fields[1].offset)
    println(type_info[Pair[int]]().name, describe[Pair[int]]())
    println(type_info[Shape]().name, len(type_info[Shape]().variants))
    a: Animal = Dog(4)
    d = Dog(3)
    println(type_info[Animal]().name, len(type_info[Animal]().fields), type_info[Dog]().name, len(type_info[Dog]().fields), a.type_info().name)
    println(d.type_info().name == "Dog")
