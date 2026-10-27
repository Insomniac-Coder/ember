#$ test: compile-fail
#$ rules: GRM-42
#$ help: say what `b` is: `for (ref mut a, ref b) in pairs:`, or `ref mut b` to change it too
## design rules: R3
## R3: the same rule for a loop's names (E0116; `V4/rules.md` section 3's message in this program's
## names).

fn main():
    pairs = [(1, 2), (3, 4)]
    for (ref mut a, b) in pairs:   #$ error[E0116]: `ref mut` here belongs to `a` only
        a += b
