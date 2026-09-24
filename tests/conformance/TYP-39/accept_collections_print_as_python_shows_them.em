#$ test: run-pass
#$ rules: TYP-39
#$ profiles: debug, release, shipping
#$ stdout: [1, 2, 3]
#$ ['a', "it's"] ['x']
#$ [[1], []] [1.5, 2.0]
#$ (1, 'x') (7,)
#$ Some(3) None Some('hi')
#$ Ok(1) Err('bad')
#$ [true, false] [1, 2, 3]
#$ got [1, 2] and (1, 'x'); again [1, 2]
#$ small=[1, 2]
# `[TYP-39]` — collections, tuples, `Option` and `Result` print as Python's
# `str()` shows them, each element by its `Debug` (text quoted).

fn parse(text: str) -> Result[int, str]:
    if text == "1":
        return Ok(1)
    return Err("bad")

fn main():
    xs = [1, 2, 3]
    println(xs)
    names: Array[String] = ["a", "it's"]
    chars: Array[char] = ['x']
    println(names, chars)
    nested: Array[Array[int]] = [[1], []]
    println(nested, [1.5, 2.0])
    println((1, "x"), (7,))
    some: Option[int] = Some(3)
    none: Option[int] = None
    text: Option[String] = "hi"
    println(some, none, text)
    println(parse("1"), parse("2"))
    fixed: [bool; 2] = [true, false]
    view: Span[int] = xs
    println(fixed, view)
    pair = (1, "x")
    small = [1, 2]
    println(f"got {small} and {pair}; again {small!r}")
    println(f"{small=}")
