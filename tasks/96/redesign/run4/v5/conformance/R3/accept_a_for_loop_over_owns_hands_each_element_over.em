#$ test: run-pass
#$ rules: OWN-8
#$ stdout: 2
## design rules: R3
## R3: `owns` is written where the default is a view, as a `for` loop's source: `for x in owns xs:`
## hands the list over and each element to its round, so `kept.push(x)` hands it on.

fn main():
    xs = [String("a"), String("b")]
    kept: Array[String] = []
    for x in owns xs:
        kept.push(x)
    println(kept.len())
