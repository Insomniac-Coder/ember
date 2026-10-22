#$ test: compile-fail
#$ rules: GRM-28, GRM-21, PHIL-12
## `[GRM-28]` — what is not built yet is refused, never ignored: a local generator (`[GRM-21]`:
## `gen fn` is legal wherever `fn` is), named.

fn main():
    gen fn count(n: int) -> int:    #$ error[E0900]: a local generator is not supported yet
        for i in 0..n:
            yield i
    for v in count(2):
        println(v)
