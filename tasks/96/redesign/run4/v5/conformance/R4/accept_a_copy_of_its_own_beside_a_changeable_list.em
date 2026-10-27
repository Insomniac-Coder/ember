#$ test: run-pass
#$ rules: FN-1
#$ stdout: ann
## design rules: R4
## R4: the nearest allowed program: `first` is a `String` of its own, so `src` reaches nothing `names`
## owns.

fn refill(ref mut dst: Array[String], src: StrView):
    dst.clear()
    dst.push(String(src))

fn main():
    names = [String("ann")]
    first = copy of names[0]
    refill(ref mut names, first)
    println(names[0])
