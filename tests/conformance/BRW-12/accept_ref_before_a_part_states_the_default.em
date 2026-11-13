# Clause BRW-12.10: "and `ref` states the default, needed before a part only to view a number (`r =
# ref counts[0]`)."
#$ test: run-pass
#$ rules: BRW-12
#$ stdout: 1 1
#$ stdout: a a
#$ stdout: 5 1

fn main():
    counts = [1, 2]
    r = ref counts[0]
    c = counts[0]
    println(r, c)
    names = [String("a")]
    n = ref names[0]
    m = names[0]
    println(n, m)
    counts[0] = 5
    println(counts[0], c)
