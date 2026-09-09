#$ test: run-pass
#$ rules: OWN-4
# The escape hatch the rule names: "unless the value is reassigned before the
# next iteration on every path". It is, so the second iteration finds a value
# there and this runs.

fn consume(owned xs: Array[i32]) -> i32:
    return xs[0]

fn main():
    a: Array[i32] = Array[i32]()
    a.push(1)
    i: i32 = 0
    while i < 2:
        println(consume(a))
        a = Array[i32]()
        a.push(9)
        i = i + 1
#$ stdout: 1
#$ 9
