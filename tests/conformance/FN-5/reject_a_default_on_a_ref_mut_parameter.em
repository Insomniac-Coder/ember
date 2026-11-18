#$ test: compile-fail
#$ rules: FN-5
#$ help: take the list over, `owns xs: Array[int] = []`, or let the caller pass one: `reset(ref mut xs)`
# `[FN-5]` — a default takes no word, since it makes the value it gives: a default is written only
# for a plain or an `owns` parameter, and one on a `ref mut` parameter is `E3027`.

fn reset(ref mut xs: Array[int] = []):    #$ error[E3027]: `ref mut xs` names the caller's place, and a default would be a temporary that ends with the call
    xs.push(owns 1)

fn main():
    items = [2]
    reset(ref mut items)
    println(items)
