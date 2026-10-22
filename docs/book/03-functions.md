# 3. Functions

A function names its parameters' types and, after `->`, its result's. A function with no `->`
returns nothing.

```ember
fn area(width: float, height: float) -> float:
    return width * height

fn greet(name: str):
    println(f"hello, {name}")

fn main():
    println(area(3.0, 4.5))
    greet("Grace")
```

```output
13.5
hello, Grace
```

## Defaults and named arguments

A parameter may have a default, and any argument may be passed by name:

```ember
fn connect(host: str, port: int = 80, secure: bool = false) -> String:
    scheme = "https" if secure else "http"
    return f"{scheme}://{host}:{port}"

fn main():
    println(connect("example.org"))
    println(connect("example.org", 8080))
    println(connect("example.org", secure=true, port=443))
```

```output
http://example.org:80
http://example.org:8080
https://example.org:443
```

## Several results

A function returns several values as a tuple, and the caller can take it apart:

```ember
fn divide(a: int, b: int) -> (int, int):
    return a // b, a % b

fn main():
    quotient, remainder = divide(17, 5)
    println(quotient, remainder)
    pair = divide(9, 2)
    println(pair.0, pair.1)
```

```output
3 2
4 1
```

## Functions as values

`fn(x) => expression` is a function without a name (Python's `lambda`). A parameter's type
`fn(int) -> int` takes any function of that shape:

```ember
fn apply_twice(f: fn(int) -> int, x: int) -> int:
    return f(f(x))

fn add_ten(x: int) -> int:
    return x + 10

fn main():
    println(apply_twice(add_ten, 1))
    println(apply_twice(fn(x) => x * 3, 2))
    offset = 5
    println(apply_twice(fn(x) => x + offset, 0))
```

```output
21
18
10
```

A lambda can use the names around it (`offset` above). Its body can also be a block:

```ember
fn apply(f: fn(int) -> int, x: int) -> int:
    return f(x)

fn main():
    clamp = fn(x: int) -> int:
        if x > 100:
            return 100
        return x
    println(apply(clamp, 250), apply(clamp, 42))
```

```output
100 42
```
