#$ test: run-pass
#$ rules: UNS-1, UNS-2, UNS-5
#$ assert-c: contains("ember_alloc")
#$ assert-c: contains("ember_free")

# `[UNS-5]` â€” the raw primitives, which are the layer `Array` and `Map` will
# be written on top of. Nothing here is a collection; this is below one.

# `[UNS-1]` â€” `alloc`, `free`, `read` and `write` are callable only inside an
# `unsafe` block. Outside one they are `E3100`.
fn sum_of_squares(n: usize) -> i32:
    total: i32 = 0
    unsafe:
        p: *mut i32 = alloc[i32](n)
        i: usize = 0
        while i < n:
            k: i32 = i as i32
            write(p, i, k * k)
            i = i + 1
        i = 0
        while i < n:
            total = total + read(p, i)
            i = i + 1
        free(p, n)
    return total

# `[UNS-2]` â€” an `unsafe` block turns nothing off. It grants the operations
# above and nothing else: this array is still bounds-checked inside it, and
# still dropped at the end of the function.
fn still_checked() -> usize:
    xs: Array[i32] = Array()
    unsafe:
        xs.push(4)
        xs.push(5)
    return xs.len()

struct Pixel:
    r: u8
    g: u8
    b: u8
    a: u8

fn main():
    println(sum_of_squares(5))
    println(still_checked())
    # `size_of` is the one primitive that needs no `unsafe`: it reads nothing.
    println(size_of[i32]())
    println(size_of[Pixel]())
#$ stdout: 30
#$ 2
#$ 4
#$ 4
