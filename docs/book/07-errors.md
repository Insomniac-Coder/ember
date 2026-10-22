# 7. Errors

Ember has no exceptions. A function that can fail says so in its result: `Result[T, E]` is either
`Ok(value)` or `Err(error)`, and the caller must look at which.

```ember
fn parse_age(text: str) -> Result[int, String]:
    match text.trim().parse[int]():
        Ok(age):
            if age < 0:
                return Err(f"{age} is negative")
            return Ok(age)
        Err(_):
            return Err(f"`{text}` is not a number")

fn main():
    for text in ["42", "-3", "old"]:
        match parse_age(text):
            Ok(age):
                println("age", age)
            Err(message):
                println("error:", message)
```

```output
age 42
error: -3 is negative
error: `old` is not a number
```

## `?`

Most of the time a function that meets an error gives it back to its own caller. `?` after a
`Result` does that: it is the value inside an `Ok`, and it returns an `Err` from the function at
once.

```ember
fn parse_age(text: str) -> Result[int, String]:
    age = text.trim().parse[int]().map_err(fn(e) => f"`{text}`: {e}")?
    if age < 0:
        return Err(f"{age} is negative")
    return Ok(age)

fn total_age(texts: Array[str]) -> Result[int, String]:
    total = 0
    for text in texts:
        total += parse_age(text)?
    return Ok(total)

fn main():
    println(total_age(["30", "12"]))
    println(total_age(["30", "x", "12"]).is_err())
```

```output
Ok(42)
true
```

A `main` may return a `Result`: an `Err` is printed and the program ends with status 1.

## Panics

Some mistakes are not errors to handle but bugs: an index past the end, a key not in a map, an
integer overflow, a failed `assert`. These **panic**: the program stops at once and prints where,
and why. `panic(message)` stops it on purpose:

```ember,fragment
fn checked_divide(a: int, b: int) -> int:
    assert(b != 0, "divide by zero")
    return a // b
```

When a value may be missing as part of normal use, give the caller an `Option` or a `Result`
instead of panicking.
