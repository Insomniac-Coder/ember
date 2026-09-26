#$ test: run-pass
#$ rules: STR-5, OWN-8
#$ stdout: true true
# The outer structural clone must keep the inner implicit clone body.
struct Leaf:
    text: String

fn main():
    x: Option[Leaf] = Some(Leaf("abc"))
    y = x.clone()
    xs: Array[Option[Leaf]] = [x]
    ys = xs.clone()
    println(y == xs[0], xs == ys)
