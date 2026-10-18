#$ test: run-pass
#$ rules: UNS-3
#$ stdout: 0 1 4 9
#$ stdout: 14
# `[UNS-3]` — no `L3010` when every statement of an `unsafe` block needs `unsafe`: a binding is
# declared before the block and assigned in it, a loop counts as needing `unsafe` when its body
# does, and the work that needs none stays outside.

fn squares(n: usize) -> i32:
    p: *mut i32
    i: usize = 0
    total: i32 = 0
    unsafe:  # SAFETY: `p` holds `n` slots, each written before it is read, and is freed once
        p = alloc[i32](n)
        while i < n:
            k: i32 = i as i32
            write(p, i, k * k)
            i = i + 1
        print(read(p, 0), read(p, 1), read(p, 2), read(p, 3))
        total = read(p, 1) + read(p, 2) + read(p, 3)
        free(p, n)
    println()
    return total

fn main():
    println(squares(4))
