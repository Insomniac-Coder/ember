#$ test: compile-fail
#$ rules: MOD-1, MNG-1, TYP-1
#$ help: rename the type
# ODR-201 — a type the program declares under a built-in type's name (`[TYP-1]`'s scalar table) is
# `E1032`: wherever a type is written the name means the built-in, and a root module's `i32` would
# have the built-in's path, so its methods the built-in's symbols (`[MNG-1]`). Before, each was
# accepted and unusable as a type. A prelude alias (`int`, `float`, `Never`) and `str` are names a
# declaration may shadow (`[MOD-5]`).

struct i32:  #$ error[E1032]: type `i32` has the name of a built-in type
    n: int

enum bool:  #$ error[E1032]: type `bool` has the name of a built-in type
    Yes
    No

class f64:  #$ error[E1032]: type `f64` has the name of a built-in type
    x: int

interface char:  #$ error[E1032]: type `char` has the name of a built-in type
    fn code(self) -> int

type usize = int  #$ error[E1032]: type `usize` has the name of a built-in type

fn main():
    println(1)
