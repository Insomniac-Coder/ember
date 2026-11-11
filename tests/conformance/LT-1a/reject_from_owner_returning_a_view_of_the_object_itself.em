#$ test: compile-fail
#$ rules: LT-1a
#$ help: return a view of `owner`, or remove `from owner`
# `[LT-1a]` — under `-> ref int from owner`, `return ref this.at` (`at` a number) is a view of the
# object itself, which no `from` names: `E3062`, and the help gives both ways out.

struct Cursor:
    owner: ref Array[int]
    at: int

    fn read() -> ref int from owner:
        return ref this.at    #$ error[E3062]: `read` returns a view of `this`, but its first line says `from owner`

fn main():
    xs = [1, 2]
    c = Cursor(ref xs, 0)
    println(c.read())
