#$ test: run-pass
#$ rules: TYP-32, GRM-31
#$ stdout: [2, 4]
#$ stdout: [1, 3, 5]
#$ stdout: a
#$ stdout: b
# `[TYP-32]` — a method returns `some I` as a function does: one in a type's body, one in an
# `extend` block, and one of a generic type, whose instances each hold their own iterator (here over
# `str`, its `next` built in).

struct Bag:
    items: Array[int]

    fn evens() -> some Iterator[Item = int]:
        return this.items.iter().copy_each().filter(fn(x) => x % 2 == 0)

extend Bag:
    fn odds() -> some Iterator[Item = int]:
        return this.items.iter().copy_each().filter(fn(x) => x % 2 == 1)

[T]
struct Shelf:
    items: Array[T]

    fn each() -> some Iterator[Item = ref T]:
        return this.items.iter()

fn main():
    b = Bag(items=[1, 2, 3, 4, 5])
    println(b.evens().to_array())
    println(b.odds().to_array())
    s = Shelf[StrView](items=["a", "b"])
    for x in s.each():
        println(x)
