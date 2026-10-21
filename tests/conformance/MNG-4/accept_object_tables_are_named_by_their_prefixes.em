#$ test: run-pass
#$ rules: MNG-4
#$ assert-c: contains("struct em_obj_Animal")
#$ assert-c: contains("em_vt_Animal")
#$ assert-c: contains("em_ti_Animal")
#$ assert-c: contains("em_ti_Dog")
#$ stdout: 4 woof
## `[MNG-4]` — a class's object struct, its method table and its type information are named
## `em_obj_`, `em_vt_` and `em_ti_` followed by the type's name as the C names it.

open class Animal:
    legs: int

    virtual fn sound(self) -> str:
        return "..."

class Dog(Animal):
    override fn sound(self) -> str:
        return "woof"

fn main():
    a: Animal = Dog(4)
    println(a.legs, a.sound())
