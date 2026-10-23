#$ test: run-pass
#$ rules: TYP-38, GRM-27, TXT-9
#$ profiles: debug, development, release
#$ stdout: {'one': 3, 'three': 5} {'one': 2, 'two': 3} 2 three {'a': 1}
# ODR-267 (D-881) — a collection written with no type takes its text type from what it holds: a
# key, value or element the program has as `str` stays `str`, so `{name: name.char_count() for _,
# name in pairs}` is a `Map[str, int]`, as `[name for _, name in pairs]` is an `Array[str]`, and a
# string literal beside it is a `str` too; only string literals make owned text (`{"a": 1}` is a
# `Map[String, int]`). No text is converted (`[TXT-9]`).

fn main():
    pairs = [(1, "one"), (2, "three")]
    sizes = {name: name.char_count() for _, name in pairs}
    first: str = pairs[0].1
    mixed = {first: 2, "two": 3}
    seen = {name for _, name in pairs}
    by_number = {k: name for k, name in pairs}
    literals = {"a": 1}
    of_str: Map[str, int] = sizes
    also: Map[str, int] = mixed
    set_of_str: Set[str] = seen
    values: Map[int, str] = by_number
    owned_text: Map[String, int] = literals
    println(of_str, also, set_of_str.len(), values[2], owned_text)
