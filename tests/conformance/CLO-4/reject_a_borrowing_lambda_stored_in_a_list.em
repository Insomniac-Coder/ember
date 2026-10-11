#$ test: compile-fail
#$ rules: CLO-4, TYP-15, TYP-23, DIA-2

# `[CLO-4]` (D-556, D-555) — a list given its element type by a lambda pushed
# into it holds that lambda, the same one on every pass over the body; one that
# borrows is a view, which a list may not hold: `E3026` (shape B9, ODR-103), whose help is
# `owned fn`.

fn main():
    fs = []
    for i in 0..3:
        s = f"item {i}"
        fs.push(fn() => s.len())    #$ error[E3026]: this lambda borrows what it captures, so it may not be stored in an `Array` unless it is `static`
    println(fs.len())
