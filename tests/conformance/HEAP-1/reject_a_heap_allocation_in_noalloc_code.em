#$ test: compile-fail
#$ rules: HEAP-1
## `[HEAP-1]` — every heap type allocates through the runtime's allocator, or the memory manager its
## type names, and carries the `Alloc` effect where it allocates: a growth of an `Array` in
## `@noalloc` code is `E4001`.

@noalloc
fn grow(ref mut xs: Array[int]):
    xs.push(owns 1)    #$ error[E4001]: reaches an allocation

fn main():
    xs: Array[int] = []
    grow(ref mut xs)
    println(xs.len())
