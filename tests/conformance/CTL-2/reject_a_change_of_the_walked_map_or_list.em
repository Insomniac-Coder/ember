#$ test: compile-fail
#$ rules: CTL-2, BRW-18
# `[CTL-2]` — while a loop walks a collection, nothing changes it but the loop's own handle: `counts[k] *= 2`
# inside `for k, v in counts:` and `xs[0] = 5` inside `for x in xs:` are `E3020`.

fn main():
    counts = Map[String, int]()
    counts.insert(owns String("a"), owns 1)
    for k, _v in counts:
        counts[k] *= 2    #$ error[E3020]
    xs = [1, 2]
    for _x in xs:
        xs[0] = 5    #$ error[E3020]
