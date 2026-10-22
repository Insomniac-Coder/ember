# 2. Values and control flow

## Names

Assigning to a new name declares it. Its type is the value's, and it cannot change:

```ember
count = 3
price = 2.5
ready = true
greeting = "hi"
count = count + 1
println(count, price, ready, greeting)
```

```output
4 2.5 true hi
```

A type can be written after the name when you want to say it, or when the value alone does not
decide it:

```ember
small: i32 = 7
ratio: f32 = 0.5
println(small, ratio)
```

```output
7 0.5
```

## Numbers

`int` is a 64-bit integer (also called `i64`) and `float` a 64-bit float (`f64`); the sized ones
are `i8` … `i128`, `u8` … `u128`, `f16`, `f32`. Arithmetic never mixes them silently: `as`
converts.

```ember
a = 7
b = 2
println(a + b, a - b, a * b, a ** b)
println(a // b, a % b, -7 // 2, -7 % 2)
println(a as float / b as float)
```

```output
9 5 14 49
3 1 -4 1
3.5
```

`//` and `%` round toward minus infinity, as in Python. `/` divides floats; on integers it is an
error that says to write `//` or to convert first. An integer that overflows its type stops the
program (it never wraps silently), and so does a division by zero.

## Booleans and comparisons

`true`, `false`, `and`, `or`, `not`; comparisons chain:

```ember
x = 5
println(1 < x < 10, x == 5 and not (x > 7), x != 5 or x >= 5)
```

```output
true true true
```

## `if`

```ember
fn describe(n: int) -> str:
    if n < 0:
        return "negative"
    elif n == 0:
        return "zero"
    else:
        return "positive"

fn main():
    println(describe(-3), describe(0), describe(8))
    big = "yes" if 8 > 5 else "no"
    println(big)
```

```output
negative zero positive
yes
```

A name assigned in every branch of an `if` can be used after it:

```ember
temperature = 31
if temperature > 30:
    advice = "stay inside"
else:
    advice = "go out"
println(advice)
```

```output
stay inside
```

## Loops

`for` walks over anything iterable — a range, a list, a string's characters. `a..b` counts from
`a` up to, not including, `b`; `range(n)` is `0..n`.

```ember
total = 0
for i in 1..5:
    total += i
println(total)

for i in range(3):
    print(i, "")
println()

n = 10
while n > 1:
    n = n // 2
println(n)
```

```output
10
0 1 2 
1
```

`break` leaves a loop and `continue` goes on to its next turn. A loop's `else` runs when it ends
on its own, without a `break`:

```ember
for n in [3, 5, 7]:
    if n % 2 == 0:
        println("found an even number")
        break
else:
    println("no even number")
```

```output
no even number
```
