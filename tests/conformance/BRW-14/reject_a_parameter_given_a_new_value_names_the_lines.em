#$ test: compile-fail
#$ rules: BRW-14, DIA-25
# `[BRW-14]` (N174) - `=` on a parameter is `E3023`, and the message gives the local of your own, `new_n`
# (`new_n_2` where the name is taken), made by the refused line itself where it is a plain `=` at the top
# level of the body and no line above reads the parameter, else by a new first line of the body, with
# each later use named by its line.

fn countdown(n: int):
    println("go")
    while n > 0:
        println(n)
        n -= 1    #$ error[E3023]: `n` is the number the caller passed and stays that number for the whole call; for a number of your own write `new_n = n` above line 9, as the first line of `countdown`'s body, and `new_n` for `n` in lines 10, 11, 12 and 13
    println(n)

fn trim_it(line: StrView) -> int:
    line = line.trim()    #$ error[E3023]: `line` is the text the caller passed and stays that text for the whole call; for a text of your own write `new_line = line.trim()` in place of line 16, and `new_line` for `line` in line 17
    return line.len()

fn taken(n: int) -> int:
    new_n = n + 1
    n = new_n * 2    #$ error[E3023]: `n` is the number the caller passed and stays that number for the whole call; for a number of your own write `new_n_2 = n` above line 20, as the first line of `taken`'s body, and `new_n_2` for `n` in lines 20, 21 and 22
    return n

fn main():
    countdown(3)
    println(trim_it(" a "), taken(1))
