#$ test: compile-fail
#$ rules: FN-1
#$ error[E3013]: cannot move out of a borrowed parameter
# `[FN-1]` — `a: A` reads: it hands nothing over out of the caller's value; the caller keeps it.

struct Player:
    name: String

fn grab(p: Player, ref mut names: Array[String]):
    names.push(owns p.name)

fn main():
    names: Array[String] = []
    grab(Player(name="a"), ref mut names)
