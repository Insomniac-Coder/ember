#$ test: compile-fail
#$ rules: OWN-9, EXP-6, OWN-3
# `[OWN-9]`, `[EXP-6]` — `owns` hands a number over as it does any value: out of a name that only
# views it (a plain parameter, a loop's name, a `ref` local) that is `E3013`, and out of an element
# `E3011`, as for a `String`.

fn take(owns x: int):
    println(x)

fn pass_on(n: int):
    take(owns n)    #$ error[E3013]

fn main():
    xs = [1, 2]
    for x in xs:
        take(owns x)    #$ error[E3013]
    r = ref xs[1]
    take(owns r)    #$ error[E3013]
    take(owns xs[0])    #$ error[E3011]
