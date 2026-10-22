#$ test: run-pass
#$ rules: GRM-17, LEX-6a, CLO-2
#$ profiles: debug, development, release
#$ stdout: 6
#$ 10
#$ hi
#$ 3
## `[GRM-17]` (ODR-209) — a lambda's `:` body written on its own line is one simple statement,
## and the statement's newline ends both it and the statement that holds the lambda:
## `h = fn(x): total += x` is one statement, and the next line is the next one. It did not
## parse: the body took the newline, and the assignment found the next line's first token (D-852).
## The loop's lambda writes one capture and reads another: the second was not found (`E1010`).

fn feed[F: fn(int)](owned f: F, n: int):
    for i in range(n):
        f(i)

fn main():
    total = 0
    h = fn(x: int): total += x
    feed(h, 4)
    println(total)
    double = fn(x: int) -> int: return x * 2
    println(double(5))
    shout = fn(): println("hi")
    shout()
    seen = 0
    for i in range(3):
        note = fn(x: int): seen += x + i
        feed(note, 1)
    println(seen)
