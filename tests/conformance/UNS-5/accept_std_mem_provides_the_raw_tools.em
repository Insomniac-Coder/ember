#$ test: run-pass
#$ rules: UNS-5
#$ stdout: 1 4
#$ stdout: 1 1 2 3
#$ stdout: 1.0
#$ stdout: moved
#$ stdout: 0 0.0 0
#$ stdout: 7
#$ stdout: 9
# `[UNS-5]` — `std.mem` provides `copy_nonoverlapping` and `copy` (which may overlap), which read and
# write through the raw pointers they are given; `transmute[A, B]`, which gives a value's bits as a `B`;
# `zeroed[T]()` for a `Zeroable` `T`; `MaybeUninit`; and `Volatile[*T]`'s volatile reads and writes.

from std.mem import copy, copy_nonoverlapping, transmute, zeroed, Volatile, MaybeUninit

@derive(Zeroable)
struct Pt:
    x: i32
    y: f64

fn main():
    a: Array[i32] = [1, 2, 3, 4]
    b: Array[i32] = [0, 0, 0, 0]
    # SAFETY: both hold four `i32`s, and they are two arrays
    unsafe:
        copy_nonoverlapping(a.as_span().as_ptr(), b.as_mut_span().as_mut_ptr(), 4)
    println(b[0], b[3])
    p = a.as_mut_span().as_mut_ptr()
    # SAFETY: three `i32`s from the first to the second, all inside `a`
    unsafe:
        copy(p as *i32, p.add(1), 3)
    println(a[0], a[1], a[2], a[3])
    # SAFETY: an `f32` and a `u32` are both four bytes, and every `u32` is a valid `f32`
    unsafe:
        println(transmute[u32, f32](owns 0x3f800000))
    s = String("moved")
    # SAFETY: a `String` is a `String`: the value is handed over whole
    unsafe:
        println(transmute[String, String](owns s))
    z: Pt = zeroed[Pt]()
    n: i64 = zeroed[i64]()
    println(z.x, z.y, n)
    slot = MaybeUninit[i32].uninit()
    slot.write(owns 7)
    # SAFETY: the slot is written before it is read
    unsafe:
        println(slot.assume_init())
    c: Array[i32] = [0]
    v = Volatile.new(c.as_mut_span().as_mut_ptr())
    # SAFETY: `c` holds one `i32`, which nothing else reaches while it is written and read
    unsafe:
        v.write_volatile(9)
        println(v.read_volatile())
