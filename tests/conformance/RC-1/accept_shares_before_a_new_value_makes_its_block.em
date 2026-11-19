#$ test: run-pass
#$ rules: RC-1, RC-7
#$ profiles: debug, development, release
#$ stdout: 6
#$ 0
#$ end 1
#$ done
# `[RC-1]` — `shares mut` before a new value makes the value in a block of its own, with the header
# of `[OBJ-1]`; another owner of it is one more count, each change through either owner is seen
# through the other with nothing checked while the program runs (`[RC-7]`), and the last owner's end
# ends the value.

struct Counter:
    value: int

struct Token:
    v: int

    fn drop():
        println(f"end {this.v}")

fn program():
    shared = shares mut Counter(value=1)
    alias = shares mut shared
    shared.value += 1
    alias.value += 1
    println(shared.value + alias.value)
    first = shares Token(v=1)
    second = shares first
    println(second.v - first.v)

fn main():
    program()
    println("done")
