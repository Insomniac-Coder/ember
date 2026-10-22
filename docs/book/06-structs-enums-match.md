# 6. Structs, enums and `match`

## Structs

A `struct` is a value made of named fields, each with its type. Its name called as a function
makes one, with the fields in order or by name; a field may have a default. Functions written in
its body are its methods, whose first parameter is `self`:

```ember
struct Point:
    x: float
    y: float
    label: String = "p"

    fn distance_to(self, other: Point) -> float:
        dx = self.x - other.x
        dy = self.y - other.y
        return (dx * dx + dy * dy).sqrt()

fn main():
    a = Point(0.0, 0.0)
    b = Point(x=3.0, y=4.0, label="b")
    println(a.distance_to(b), b.label)
    println(b)
```

```output
5.0 b
Point(x=3.0, y=4.0, label='b')
```

A struct is printed field by field, compared with `==` field by field, and copied with `clone()`,
with nothing to write. A struct is a value: assigning it to another name, or passing it on, moves it
(chapter 8).

## Enums

An `enum` is one of several variants, each of which may carry values:

```ember
enum Shape:
    Circle(radius: float)
    Rectangle(width: float, height: float)
    Empty

fn area(shape: Shape) -> float:
    match shape:
        Shape.Circle(r):
            return 3.14159 * r * r
        Shape.Rectangle(w, h):
            return w * h
        Shape.Empty:
            return 0.0

fn main():
    shapes = [Shape.Circle(1.0), Shape.Rectangle(2.0, 3.0), Shape.Empty]
    for shape in shapes:
        println(area(shape))
```

```output
3.14159
6.0
0.0
```

`match` tries each pattern in order and runs the first that fits, binding the values a variant
carries to the names in it. The compiler checks that the patterns cover every variant: leaving one
out is an error, which names it.

## Patterns

Patterns also match numbers, ranges of them, text, tuples, and anything with `_`:

```ember
fn classify(n: int) -> str:
    match n:
        0:
            return "zero"
        1 | 2 | 3:
            return "small"
        _:
            return "large"

fn main():
    println(classify(0), classify(2), classify(40))
    point = (0, 7)
    match point:
        (0, y):
            println("on the y axis at", y)
        (x, _):
            println("x is", x)
```

```output
zero small large
on the y axis at 7
```

## `Option`

A value that may be missing is an `Option`: `Some(value)` or `None`. There is no null; a list's
`first` and `last`, a map's `get` and a string's `find` give one:

```ember
fn main():
    xs = [4, 8, 15]
    match xs.last():
        Some(v):
            println("last is", v)
        None:
            println("empty")
    empty: Array[int] = []
    println(empty.first(), xs.first())
    position = "hay needle".find("needle")
    if Some(at) = position:
        println("found at", at)
    println(position.unwrap_or(-1), "x".find("y").unwrap_or(-1))
    if empty.first() is None:
        println("nothing first")
```

```output
last is 15
None Some(4)
found at 4
4 -1
nothing first
```

`if Some(at) = position:` runs its block when the pattern fits, binding `at`; `is None` tests for
the missing case.
