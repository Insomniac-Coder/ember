#$ test: compile-fail
#$ rules: GRM-30, TYP-6
#$ profiles: debug
# `[GRM-30]` — `as?` and `as!` are not Ember (`E1010`): the message gives the kind test for a downcast
# that may fail and `cast s as Circle` for one that stops the program; `cast s when D` is none either.

open class Animal:
    fn name() -> int:
        return 0

class Dog(Animal):
    fn bark() -> int:
        return 1

fn main():
    a: Animal = Dog()
    d = a as! Dog    #$ error[E1010]: `as!` is not Ember: write `cast a as Dog`
    m = a as? Dog    #$ error[E1010]: `as?` is not Ember: test the kind, `if a is Dog as c:`
    w = cast a when Dog    #$ error[E1010]: `cast a when Dog` is not Ember
