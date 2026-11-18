#$ test: run-pass
#$ rules: TYP-39
#$ stdout: [1, 2, 3] {'a': 1} {1, 2} (1, 'x') Some(3)
#$ {} set()
#$ ["it's", 'say "hi"', 'both \' and "']
#$ 5 4
#$ ['\x01'] ['a']
## `[TYP-39]` - collections, tuples and `Option` show as Python's `str()` shows them, each element by its `Debug`
## (strings quoted: single quotes, or double quotes when the text holds a `'` and no `"`; `\`, the quote, `\n`,
## `\r`, `\t` escaped, other control characters `\xNN`); an empty map is `{}`, an empty set `set()`; an `owns T` and
## a `Cell[T]` show as the `T` they hold.

fn main():
    println([1, 2, 3], {"a": 1}, {1, 2}, (1, 'x'), Some(3))
    e: Map[int, int] = {}
    s: Set[int] = Set()
    println(e, s)
    println(["it's", "say \"hi\"", "both ' and \""])
    p: owns int = owns 5
    c = Cell(4)
    println(p, c)
    println(["\x01"], ['a'])
