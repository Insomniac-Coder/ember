#$ test: compile-fail
#$ rules: GRM-30, TYP-6
#$ profiles: debug
# ODR-280 — `as!` and `as?` are not Ember: a downcast is written `cast x as D` (the `D`, or a panic)
# or `cast x if D` (`Some(d)` or `None`), and the error for the old spelling names the new one.

open class Animal:
    fn name(self) -> int:
        return 0

class Dog(Animal):
    fn bark(self) -> int:
        return 1

fn main():
    a: Animal = Dog()
    d = a as! Dog    #$ error[E0100]: `as!` is not Ember: a downcast is written `cast x as D`
    m = a as? Dog    #$ error[E0100]: `as?` is not Ember: a downcast is written `cast x if D`
