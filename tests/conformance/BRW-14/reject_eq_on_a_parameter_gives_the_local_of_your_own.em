# Clause BRW-14.1: "A parameter that is not `owns` (...) stand[s] for what the caller (...) gave, and
# keep[s] it for their whole scope: `=` on one is `E3023` ("`line` is the text the caller passed and
# stays that text for the whole call; for a text of your own write `new_line = line.trim()` in place
# of line 2, and `new_line` for `line` in line 3"; "`n` is the number the caller passed and stays
# that number for the whole call; for a number of your own write `new_n = n` above line 2, as the
# first line of `countdown`'s body, and `new_n` for `n` in lines 3, 4 and 6": one form for every
# parameter, its noun the same in its three places)"
#$ test: compile-fail
#$ rules: BRW-14, DIA-25
# The spec's two messages, with the file's line numbers. `countdown` is laid out as the spec's line
# numbers say: its first body line does not read `n`, and lines 3, 4 and 6 of it do.

fn trim_it(line: StrView) -> int:
    line = line.trim()    #$ error[E3023]: `line` is the text the caller passed and stays that text for the whole call; for a text of your own write `new_line = line.trim()` in place of line 14, and `new_line` for `line` in line 15
    return line.len()

fn countdown(n: int):
    println("counting")
    while n > 0:
        n -= 1    #$ error[E3023]: `n` is the number the caller passed and stays that number for the whole call; for a number of your own write `new_n = n` above line 18, as the first line of `countdown`'s body, and `new_n` for `n` in lines 19, 20 and 22
        println("tick")
    println(n)

fn main():
    println(trim_it(" a "))
    countdown(2)
