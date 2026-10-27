#$ test: run-pass
#$ rules: R12, R8
#$ stdout: 1
#$ stdout: 2
#$ stdout: 4
## R8 (c), R12 (rules.md lines 717-720): the program the E3020 message of the nearest refused one leads
## to (reject_a_stored_function_that_pushes_to_the_walked_list_called_in_the_loop): the walk of `xs`
## ends, then the stored function runs as many times as the loop went round.

fn main():
    xs = [1, 2]
    hs: Array[fn()] = []
    hs.push(fn(): xs.push(0))
    n = xs.len()
    for x in xs:
        println(x)
    for k in 0..n:
        hs[0]()
    println(xs.len())
