# Clause BCK-5.1: "during reservation it conflicts only with a change of its place"; "When the call
# starts, its arguments are checked against each other: two of them, at least one changeable (a
# function value counting as changing every place its functions change), name one place or reach
# into each other only where the compiler proves them one whole variable (`[MONO-10]`) (`E3022`
# where both may change it, `E3021` otherwise)."
#$ test: compile-fail
#$ rules: BCK-5, BRW-3, BRW-11
# An argument that changes the reserved place while it is evaluated, and two arguments that reach
# into each other when the call starts. No text in the spec for these: code only.

fn take_last(ref mut xs: Array[int]) -> int:
    return xs.pop().unwrap()

fn add_to(ref mut xs: Array[int], x: int):
    xs.push(owns x)

fn add_text(ref mut xs: Array[String], s: StrView):
    xs.push(owns String(s))

fn main():
    v = [1, 2]
    add_to(ref mut v, take_last(ref mut v))    #$ error[E3022]
    w = [String("a")]
    add_text(ref mut w, w[0])    #$ error[E3021]
