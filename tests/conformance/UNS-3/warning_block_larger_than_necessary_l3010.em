#$ test: run-pass
#$ rules: UNS-3
#$ stdout: 42
#$ stdout: freeing
#$ stdout: 7
# `[UNS-3]` — `L3010` reports an `unsafe` block holding a statement that needs no `unsafe` (here the
# `println`), and one holding nothing that needs it. The lint warns and the program still runs.

fn main():
    p: *mut i32
    unsafe:  # SAFETY: one slot, written before it is read and freed once    #$ warning[L3010]: larger than necessary
        p = alloc[i32](1)
        write(p, 0, 42)
        println(read(p, 0))
        println("freeing")
        free(p, 1)
    unsafe:  # SAFETY: nothing here needs it    #$ warning[L3010]: contains no operation that needs `unsafe`
        println(7)
