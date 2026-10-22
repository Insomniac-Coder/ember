# 8. Ownership

Ember frees memory without a garbage collector, at a point you can see in the code: each value has
one **owner**, the name holding it, and when the owner goes out of scope the value is freed. Most
code never shows this. This chapter is about the times it does.

## Assigning moves

Assigning a list, a `String` or a struct to another name **moves** it: the new name owns it, and
the old one can no longer be used. The compiler says so where it happens:

```ember
fn main():
    first = [1, 2, 3]
    second = first
    println(second)
```

```output
[1, 2, 3]
```

Using `first` after `second = first` is `E3040` ("`first` is borrowed after it has been moved
out of"), whose help is to clone the value so that the move takes the copy: `second =
first.clone()` leaves each name a list of its own. Numbers, booleans and other small values are
copied rather than moved, so there is nothing to think about for them.

## Passing borrows

Passing a value to a function **borrows** it by default: the function can read it, and the caller
keeps it.

```ember
fn total(values: Array[int]) -> int:
    return sum(values)

fn main():
    xs = [1, 2, 3]
    println(total(xs))
    println(total(xs), xs)
```

```output
6
6 [1, 2, 3]
```

A parameter marked `mut` can change what it is given, and the caller writes nothing extra:

```ember
fn add_tax(mut prices: Array[float], rate: float):
    for i in range(len(prices)):
        prices[i] = prices[i] * (1.0 + rate)

fn main():
    prices = [10.0, 20.0]
    add_tax(prices, 0.5)
    println(prices)
```

```output
[15.0, 30.0]
```

A parameter marked `owned` takes the value over: the caller gives it up, as with an assignment.
That is how a value is stored somewhere that outlives the call:

```ember
struct Inbox:
    messages: Array[String] = []

    fn deliver(mut self, owned message: String):
        self.messages.push(message)

fn main():
    inbox = Inbox()
    text = "hello".to_string()
    inbox.deliver(text)
    println(inbox.messages)
```

```output
['hello']
```

## Borrowing from a value

A slice of a list (`xs[1..3]`), a `str` of a `String`, and a `ref` to an element all **borrow**
from the value they point into. While one is in use, the value cannot be changed or moved — a
change could move the very elements it points at. The compiler checks this, and says which borrow
is still in use and where it ends:

```ember
fn main():
    xs = [1, 2, 3, 4]
    window = xs[1..3]
    println(window)
    xs.push(5)
    println(xs)
```

```output
[2, 3]
[1, 2, 3, 4, 5]
```

`window` is last used before the `push`, so its borrow has ended by then and the `push` is fine.
Printing `window` after the `push` would be `E3021`.

## Why

There is no collector to pause the program, no `None` that turns up where a value was expected,
and no list changed under a loop walking it: the compiler checks each of these before the program
runs. `ember explain --borrow file.em:LINE` shows every borrow alive at a line and the use that
keeps it alive.
