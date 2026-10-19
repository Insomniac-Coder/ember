#$ test: run-pass
#$ rules: TYP-17, TYP-24
#$ profiles: debug, development, release
#$ stdout: 2 3 4 1
# D-603 — a method written in an `extend T implements I` block replaces I's
# default body for `T`, as one written in `T`'s own body does; the default is
# `T`'s only where nothing replaces it. (The default's body was also checked
# for the written method's entry: two C bodies of one symbol, or the default
# called.)

interface Greet:
    fn hello(self) -> int:
        return 1

interface Named:
    fn name(self) -> int

struct S:
    v: int

extend S implements Greet:
    fn hello(self) -> int:
        return 2

struct T:
    v: int

extend T implements Named, Greet:
    fn name(self) -> int:
        return 5

    fn hello(self) -> int:
        return 3

struct U implements Greet:
    v: int

    fn hello(self) -> int:
        return 4

struct V implements Greet:
    v: int

fn main():
    println(S(0).hello(), T(0).hello(), U(0).hello(), V(0).hello())
