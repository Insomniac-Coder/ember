#$ test: run-pass
#$ rules: CLO-1, CLO-2, LT-42, TYP-15, LT-3, TST-19
#$ stdout: left

# An `owned fn` can contain a view only when every captured view region is
# static. `str` literals provide those static regions, so this closure may
# own the Pair environment.

@view
struct Pair:
    left: StrView
    right: StrView

fn main():
    pair = Pair("left", "right")
    task = owned fn() => pair.left
    println(task())
