#$ test: compile-fail
#$ rules: DIA-7, DIA-25, FN-2
#$ error[E3023]: cannot mutate borrowed parameter `letter`
#$ help: give the value a local of your own first, `new_letter = letter`, and change and use `new_letter` from here on; or write `ref mut letter: String` for the parameter, and `ref mut` before the argument at each call, if the caller is to see the new value
# `[DIA-7]`, shape B4 (ODR-235) — the same for a parameter that is a `String`, whose whole is
# reached through its borrow: `letter = letter.to_lower()` is a rebinding, and gets the same line.

fn shout(letter: String) -> String:
    letter = letter.to_lower()
    return letter

fn main():
    println(shout("A".to_string()))
