#$ test: run-pass
#$ rules: TYP-16, STR-1, UNS-5

# `[TYP-16]` â€” a generic struct. One declaration; one instantiation per
# argument list, chosen by inference at the constructor.
struct Pair[A, B]:
    first: A
    second: B
    fn left(self) -> A:
        return self.first

# A growable collection written in Ember rather than known to the compiler.
# This is what block D builds on: `Array[T]` is this with a nicer surface and
# a `drop`.
struct Buffer[T]:
    ptr: *mut T
    len: usize
    cap: usize

fn make[T](cap: usize) -> Buffer[T]:
    unsafe:
        return Buffer(alloc[T](cap), 0, cap)

fn grow[T](mut b: Buffer[T]):
    bigger: usize = b.cap * 2
    unsafe:
        fresh: *mut T = alloc[T](bigger)
        i: usize = 0
        while i < b.len:
            write(fresh, i, read(b.ptr, i))
            i = i + 1
        free(b.ptr, b.cap)
        b.ptr = fresh
    b.cap = bigger

fn push[T](mut b: Buffer[T], v: T):
    if b.len == b.cap:
        grow(b)
    unsafe:
        write(b.ptr, b.len, v)
    b.len = b.len + 1

fn at[T](b: Buffer[T], i: usize) -> T:
    unsafe:
        return read(b.ptr, i)

fn release[T](mut b: Buffer[T]):
    unsafe:
        free(b.ptr, b.cap)
    b.cap = 0
    b.len = 0

fn main():
    # Inference at the constructor: `Pair[i32, f32]`.
    p = Pair(7, 2.5)
    println(p.left())
    println(p.second)

    # `Buffer[T]` in the signatures above unifies with `Buffer[i32]` here.
    b: Buffer[i32] = make[i32](2)
    i: i32 = 0
    while i < 9:
        push(b, i * i)
        i = i + 1
    println(b.len)
    println(b.cap)
    println(at(b, 0))
    println(at(b, 8))
    release(b)
#$ stdout: 7
#$ 2.5
#$ 9
#$ 16
#$ 0
#$ 64
