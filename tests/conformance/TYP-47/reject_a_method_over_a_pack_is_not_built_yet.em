#$ test: compile-fail
#$ rules: TYP-47
# `[TYP-47]` — functions, structs, classes and `extend` blocks over a pack are built; a method over
# a pack of its own, one taking its owner's pack's values, and an enum, an interface or a `type`
# alias over a pack are not built by this compiler yet (`E0900`), and say so rather than failing
# at their uses.

struct Log:
    n: int

    [Ts...]    #$ error[E0900]: a method over a pack of its own (`Ts...`) is not built by this compiler yet
    fn all(values: Ts...):
        pass

[Cs...]
struct Row:
    id: int

    fn put(values: Cs...):    #$ error[E0900]: a method that takes a pack's values is not built by this compiler yet
        pass

enum Either[Ts...]:    #$ error[E0900]: an enum over a pack (`Ts...`) is not built by this compiler yet
    Nothing

fn main():
    pass
