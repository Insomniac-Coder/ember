#$ test: compile-fail
#$ rules: LT-1
# Under rule 3 the caller keeps every source argument borrowed while the
# result lives, `ys` included, whatever the body returns.

@view
struct V:
    r: Span[int]

    fn m(owned self, a: Array[int]) -> Span[int]:
        return self.r

fn main():
    xs = [1, 2]
    ys = [3]
    v = V(r=xs)
    s = v.m(ys)
    ys.push(4)    #$ error[E3021]: `ys` is borrowed here and mutably borrowed elsewhere
    println(s)
