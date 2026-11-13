#$ test: compile-fail
#$ rules: LEX-21, GRM-30
#$ profiles: debug
## `[LEX-21]` - `...` is one token, not `..` and `.`; `as?` and `as!` are single tokens lexed only to be
## refused with `E1010`, whose message gives the kind test or the cast to write; `!` has no other use.

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
    r = 1 ... 3      #$ error[E0100]: found `...`
    n = !true        #$ error[E0100]: expected an expression
#$ error[E0100]: expected the end of the line
