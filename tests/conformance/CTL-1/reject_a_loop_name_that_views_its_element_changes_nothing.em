#$ test: compile-fail
#$ rules: CTL-1, BRW-14
#$ help: to change the number in `xs` itself, write `for ref mut v in xs:`
#$ help: to change the number in `counts` itself, write `for k, ref mut v in counts:`
# `[CTL-1]` — a loop name that views its element changes nothing: `for v in xs.iter_mut(): v += 1` is
# `E3023`, whose help gives `for ref mut v in xs:`; so is a value a `Map`'s two names give.

fn main():
    xs = [1, 2, 3]
    for v in xs.iter_mut():
        v += 1    #$ error[E3023]
    counts = Map[String, int]()
    for k, v in counts:
        v *= 2    #$ error[E3023]
        println(k)
