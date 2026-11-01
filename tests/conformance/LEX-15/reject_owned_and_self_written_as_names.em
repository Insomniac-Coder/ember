#$ test: compile-fail
#$ rules: LEX-15, FN-4
# `[LEX-15]` — `owned` and `self` are reserved and not words of Ember: any use is `E1010`, whose
# message gives the word to write, a use as a name too (a field, a function, a parameter, a local,
# a loop's name). `self` anywhere is "inside a method its object is `this`" (`[FN-4]`): a function
# standing alone has no object, and a `self` it lists is not dropped from its parameters.

struct S:
    owned: int    #$ error[E1010]: `owned` is gone from Ember: the word that hands a value over is `owns`

fn self() -> int:    #$ error[E1010]: inside a method its object is `this`
    return 1

fn first(self: int) -> int:    #$ error[E1010]: inside a method its object is `this`
    return 1

fn second(x: int) -> int:
    y = self    #$ error[E1010]: inside a method its object is `this`
    return x

fn main():
    owned = 3    #$ error[E1010]: `owned` is gone from Ember
    for self in [1, 2]:    #$ error[E1010]: inside a method its object is `this`
        println(first(1), second(2))
