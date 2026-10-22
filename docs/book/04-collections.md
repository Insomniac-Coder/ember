# 4. Collections

## `Array`

`[a, b, c]` makes an `Array`, a list that grows. `xs[i]` reads element `i` (from 0), `len(xs)` or
`xs.len()` is its length, and `push` adds to its end:

```ember
xs = [3, 1, 4]
xs.push(1)
xs.push(5)
println(xs, len(xs), xs[0])
xs[1] = 9
println(xs)
println(sum(xs), sorted(xs), min(xs[0], xs[1]))
```

```output
[3, 1, 4, 1, 5] 5 3
[3, 9, 4, 1, 5]
22 [1, 3, 4, 5, 9] 3
```

`min` and `max` compare two values. An index past the end stops the program with a message saying
which index and how long the list was; so does a negative index. `xs.last()` and `xs.first()` give
an `Option` instead (chapter 6).

An empty list needs its type said, since nothing in it says it:

```ember
names: Array[String] = []
names.push("Ada".to_string())
println(names.len())
```

```output
1
```

## Walking a list

```ember
words = ["red", "green", "blue"]
for word in words:
    print(word, "")
println()
for i, word in enumerate(words):
    println(i, word)
```

```output
red green blue 
0 red
1 green
2 blue
```

## Slices

`xs[a..b]` is the part from `a` up to, not including, `b`: a view of the list, not a copy.

```ember
xs = [10, 20, 30, 40, 50]
middle = xs[1..4]
println(middle, middle.len())
println(sum(xs[2..]))
```

```output
[20, 30, 40] 3
120
```

## Comprehensions

```ember
squares = [n * n for n in 1..6]
evens = [n for n in squares if n % 2 == 0]
println(squares, evens)
```

```output
[1, 4, 9, 16, 25] [4, 16]
```

## `Map`

`{key: value}` makes a `Map`. It keeps its keys in the order they were first added, like a Python
`dict`:

```ember
ages = {"ada": 36, "alan": 41}
ages["grace"] = 85
println(ages["alan"], len(ages))
println("ada" in ages, "bob" in ages)
println(ages.get_or("bob", 0))
for name in ages:
    println(name, ages[name])
for name, age in ages.items():
    println(f"{name} is {age}")
```

```output
41 3
true false
0
ada 36
alan 41
grace 85
ada is 36
alan is 41
grace is 85
```

Reading a key that is not there stops the program, as Python's `KeyError` does; `get_or` gives a
value instead, and `get` gives an `Option`.

## `Set`

```ember
seen = {3, 1, 4}
seen.add(1)
seen.add(5)
println(len(seen), 4 in seen, 2 in seen)
```

```output
4 true false
```
