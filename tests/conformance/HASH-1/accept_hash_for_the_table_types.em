#$ test: run-pass
#$ rules: HASH-1, TYP-36, ENM-3, DRV-1, STD-12
#$ stdout: true false
#$ stdout: true
#$ stdout: false
#$ stdout: true false
#$ stdout: true true
#$ stdout: false true
#$ stdout: true false
#$ stdout: false true
#$ stdout: true
#$ stdout: true true
#$ stdout: false
# `[TYP-36]` — the table's `Hash` column: `char`, text (a `String` exactly as
# its `str`, `[STD-12]`), tuples and fixed arrays of hashable parts, `Array`,
# `Option`, `Result`, unit-only enums (`[ENM-3]`) and `@derive(Hash)` structs
# and enums (`[DRV-1]`: fields in order, a variant's index first). Equal
# values hash equally (`[HASH-1]`); these unequal ones happen to differ.

from std.collections import Hash, Hasher, DefaultHasher

@derive(Hash)
struct P:
    x: i32
    name: String

@derive(Hash)
enum Shape:
    Circle(r: int)
    Rect(w: int, h: int)
    Empty

enum Color:
    Red
    Blue

fn hash_of[K: Hash](k: K) -> u64:
    h = DefaultHasher.new()
    k.hash(h)
    return h.finish()

fn hash_with[K: Hash, H: Hasher + Default](k: K) -> u64:
    h = H.default()
    k.hash(h)
    return h.finish()

fn main():
    println(hash_of('a') == hash_of('a'), hash_of('a') == hash_of('b'))
    println(hash_of("ab") == hash_of(String.from("ab")))
    println(hash_of(("ab", "c")) == hash_of(("a", "bc")))
    println(hash_of((1, 2)) == hash_of((1, 2)), hash_of((1, 2)) == hash_of((2, 1)))
    println(hash_of([1, 2]) == hash_of([1, 2]), hash_of(Some(3)) == hash_of(Some(3)))
    println(hash_of(Color.Red) == hash_of(Color.Blue), hash_of(Color.Red) == hash_of(Color.Red))
    println(hash_of(P(1, "a")) == hash_of(P(1, "a")), hash_of(P(1, "a")) == hash_of(P(1, "b")))
    println(hash_of(Shape.Circle(1)) == hash_of(Shape.Rect(1, 0)), hash_of(Shape.Empty) == hash_of(Shape.Empty))
    fixed: [int; 3] = [1, 2, 3]
    println(hash_of(fixed) == hash_of([1, 2, 3]))
    ok: Result[int, String] = Ok(1)
    println(hash_of(ok) == hash_of(ok), hash_with[int, DefaultHasher](7) == hash_of(7))
    println(hash_of("the quick brown fox jumps") == hash_of("the quick brown fox jumpt"))
