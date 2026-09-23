#$ test: run-pass
#$ rules: GRM-38, STD-26
#$ profiles: debug, release, shipping
#$ stdout: 55 true false
#$ 30 true
# As the only argument of a call a generator expression needs no parentheses
# of its own: `sum(x * x for x in xs)`.

fn main():
    xs = [1, 2, 3, 4, 5]
    println(sum(x * x for x in xs), any(x > 4 for x in xs), all(x > 1 for x in xs))
    println(sum((x for x in range(5) if x % 2 == 0), start=24), any(a + b == 7 for a in xs for b in xs))
