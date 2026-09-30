#$ test: compile-fail
#$ rules: STR-2, CLS-3, DIA-14
# D-393 — a class's field default is read where the class is declared, as a
# struct's is (`[STR-2]`): the constructing function's `scale` is not in
# scope. It was, so this printed 42. One mistake in a default is one error,
# however many constructions evaluate it (`[DIA-14]`).

class Acc:
    k: int = scale * 2          #$ error[E1010]: cannot find `scale` in this scope

fn main():
    scale = 21
    a = Acc()
    b = Acc()
    println(a.k, b.k)
