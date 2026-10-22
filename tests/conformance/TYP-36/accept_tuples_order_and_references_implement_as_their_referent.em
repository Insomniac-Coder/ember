#$ test: run-pass
#$ rules: TYP-36, TYP-37, TYP-17
#$ profiles: debug, development, release
#$ stdout: [(1, 'a'), (1, 'z'), (2, 'a'), (2, 'b')]
#$ true Greater true Equal true
#$ [(1, 'y'), (3, 'x')]
#$ [(0, Version(major=9, minor=9)), (1, Version(major=1, minor=5)), (1, Version(major=2, minor=0))]
#$ false true Less false true Less
#$ false true
#$ apple 'apple'
#$ true false
#$ Some('c')
## `[TYP-36]` — a tuple is `Ord` lexicographically when every element is: the first elements
## decide unless equal, then the next (it was `E2040`, D-820). And (ODR-181) a shared reference
## implements `Eq`, `Ord`, `Hash`, `Display` and `Debug` as what it refers to, so a generic
## function bounded by them takes `ref x`, and compares, hashes and prints the referent. An element
## with an `Ord` of its own is compared by its `cmp`.

import std.collections

@derive(Ord)
struct Version:
    major: int
    minor: int

fn bigger[T: Ord](a: T, b: T) -> bool:
    return a > b

fn order[T: Ord](a: T, b: T) -> Ordering:
    return a.cmp(b)

fn same[T: Eq](a: T, b: T) -> bool:
    return a == b

fn show[T: Display + Debug](a: T) -> String:
    return f"{a} {a!r}"

fn digest[T: Hash](a: T) -> u64:
    h = collections.DefaultHasher.new()
    a.hash(h)
    return h.finish()

fn main():
    pairs: Array[(int, String)] = [(2, "b"), (1, "z"), (2, "a"), (1, "a")]
    pairs.sort()
    println(pairs)
    println(pairs[0] < pairs[1], pairs[2].cmp(pairs[1]), bigger(pairs[3], pairs[2]), order((1, 2.5), (1, 2.5)), ((1, "a"), 2) < ((1, "b"), 0))
    println(sorted([(3, 'x'), (1, 'y')]))
    versions = [(1, Version(2, 0)), (1, Version(1, 5)), (0, Version(9, 9))]
    versions.sort()
    println(versions)
    x = "apple".to_string()
    y = "banana".to_string()
    a = ref x
    b = ref y
    println(a == b, a < b, a.cmp(b), bigger(a, b), bigger(b, a), order(a, b))
    println(same(a, b), same(a, a))
    println(show(a))
    println(digest(a) == digest(ref x), digest(a) == digest(b))
    names: Array[String] = ["b", "c", "a"]
    println(names.iter().max())
