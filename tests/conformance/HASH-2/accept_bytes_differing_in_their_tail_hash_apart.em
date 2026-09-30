#$ test: run-pass
#$ rules: HASH-2, HASH-1
#$ stdout: true true true true
# D-405 — both hashers take bytes eight to a word, and the last one to seven
# bytes are one word with their count in its top byte. Without the count,
# `"a"` and `"a\0"` were one word and hashed alike under every key, so a
# `RandomState` map could be given keys that collide whatever its seed.

from std.collections import DefaultHasher, RandomState

fn fixed(s: str) -> u64:
    h = DefaultHasher.new()
    s.hash(h)
    return h.finish()

fn seeded(s: str) -> u64:
    h = RandomState.new()
    s.hash(h)
    return h.finish()

fn main():
    println(fixed("a") != fixed("a\0"), seeded("a") != seeded("a\0"), fixed("abcdefgh") != fixed("abcdefgh\0"), seeded("ab\0\0") != seeded("ab\0"))
