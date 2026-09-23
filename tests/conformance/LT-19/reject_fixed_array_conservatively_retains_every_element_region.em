#$ test: compile-fail
#$ rules: LT-17, LT-19, TST-17

# Fixed arrays intentionally use one conservative region slot rather than
# metadata proportional to their source-level length. Every element's
# provenance must still feed that slot; otherwise `first = 3` would be
# accepted while `refs` retained a live reference to `first`.
fn main():
    first: i32 = 1
    second: i32 = 2
    refs: [ref i32; 2] = [ref first, ref second]
    println(refs[0])
    first = 3 #$ error[E3021]: `first` cannot be written while it is borrowed
    println(refs[1])
