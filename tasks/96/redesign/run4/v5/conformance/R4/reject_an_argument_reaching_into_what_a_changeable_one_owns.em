#$ test: compile-fail
#$ rules: BRW-11
#$ help: first = copy of names[0]
## design rules: R4
## R4: an argument may not reach into what a changeable argument of the same call owns: `names[0]`'s
## text belongs to `names`, which `refill` may clear (rules section 3, E3021).

fn refill(ref mut dst: Array[String], src: StrView):
    dst.clear()
    dst.push(String(src))

fn main():
    names = [String("ann")]
    refill(ref mut names, names[0])    #$ error[E3021]: `refill` may change `names` through `dst` while `src` shows the text of `names[0]`, which `names` owns; give `src` a copy of its own: instead of this line write `first = copy of names[0]` and `refill(ref mut names, first)`
    println(names[0])
