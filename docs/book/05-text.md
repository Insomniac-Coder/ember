# 5. Text

Text is UTF-8. A string literal is a `str`: a view of text that something else holds (here, the
program). A `String` owns its text and can grow. `to_string()` makes a `String` from a `str`, and
a `String` is used wherever a `str` is asked for:

```ember
fn shout(text: str) -> String:
    return text.to_upper() + "!"

fn main():
    name = "ember"
    full = name.to_string()
    full.push_str(" lang")
    println(shout(name), shout(full))
```

```output
EMBER! EMBER LANG!
```

## f-strings

`f"…{expression}…"` puts values into text. A format after `:` aligns, pads and rounds as in
Python; `=` shows the expression too, and `!r` the value as code would write it:

```ember
item = "tea"
price = 3.14159
count = 7
println(f"{item}: {price:.2f} x {count}")
println(f"[{item:>6}] [{count:03}] [{price:8.3f}]")
println(f"{count=} {item!r}")
```

```output
tea: 3.14 x 7
[   tea] [007] [   3.142]
count=7 'tea'
```

## Length

`s.len()` is the length in bytes, which is what slicing counts. `s.char_count()` counts characters,
which is what Python's `len` does. `len(s)` is refused for a string, so that neither is taken for
the other:

```ember
word = "naïve"
println(word.len(), word.char_count())
```

```output
6 5
```

## Splitting, trimming, joining

`split` and `lines` give their parts one at a time, for a `for` or a comprehension; a list of text
prints as Python prints one:

```ember
line = "  red, green ,blue  "
parts = [part.trim() for part in line.split(",")]
println(parts)
println(parts.join(" | "))
println([word for word in "a b  c".split_whitespace()])
println([text for text in "one\ntwo".lines()])
```

```output
['red', 'green', 'blue']
red | green | blue
['a', 'b', 'c']
['one', 'two']
```

## Searching

`find` gives the position of a match as an `Option`, since there may be none (chapter 6 shows how
to take one apart):

```ember
text = "the cat sat"
println(text.find("cat"), text.find("dog"))
println(text.starts_with("the"), text.ends_with("sat"), "cat" in text)
println(text.replace("cat", "dog"))
```

```output
Some(4) None
true true true
the dog sat
```

## Parsing numbers

`s.parse[int]()` reads a number from text. It gives a `Result`, because the text may not be one
(chapter 7):

```ember
fn main():
    match "42".parse[int]():
        Ok(n):
            println(n + 1)
        Err(e):
            println("not a number:", e)
    println("4x2".parse[int]().is_err())
```

```output
43
true
```
