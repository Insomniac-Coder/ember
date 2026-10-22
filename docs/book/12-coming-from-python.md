# 12. Coming from Python

Ember reads like Python and runs like C. Most Python habits carry over unchanged; the ones that do
not are rejected with an error that names the Ember way, rather than given a different meaning.
This chapter follows Appendix E of the specification, with every claim a program the guide's tests
run.

## What carries over

Indentation and `:` blocks, `#` comments, `x = 1` declaring, tuples and unpacking, list, map and set
literals and comprehensions, `for … else`, `enumerate`, `len`, `range`, `sum`, `sorted`, `zip`,
`reversed`, `any`, `all`, `in`, chained comparisons, `is None`, floor division, `**`, f-strings with
`=`, `!r` and format specs, `print(…, sep=…, end=…)`, `input`, keyword arguments and defaults:

```ember
pairs = [(1, "one"), (2, "two"), (3, "three")]
for number, name in pairs:
    print(number, name, sep="=", end="; ")
println()
names = [name for _, name in pairs if name.char_count() > 0 and name != "two"]
println(names, len(names), "one" in names)
lengths = {number: name.char_count() for number, name in pairs}
println(lengths)
words = ["a", "b", "c"]
for i, word in enumerate(reversed(words)):
    print(i, word, "")
println()
println(sum(range(5)), sorted([3, 1, 2]), any([false, true]), all([true, false]))
println(-7 // 2, -7 % 2, 2 ** 10, 1 < 2 < 3)
total = 3.14159
println(f"{total:.2f} {total=:.1f}")
```

```output
1=one; 2=two; 3=three; 
['one', 'three'] 2 true
{1: 3, 2: 3, 3: 5}
0 c 1 b 2 a 
10 [1, 2, 3] true false
-4 1 1024 true
3.14 total=3.1
```

A map keeps its keys in the order they were added, as a `dict` does, and iterating it gives its keys:

```ember
stock = {"apples": 3, "pears": 0}
stock["plums"] = 7
for fruit in stock:
    print(fruit, "")
println()
for fruit, count in stock.items():
    if count > 0:
        println(fruit, count)
```

```output
apples pears plums 
apples 3
plums 7
```

## Names that differ

| Python | Ember |
|---|---|
| `len(s)` for a string | `s.char_count()` (characters, Python's count) or `s.len()` (bytes) |
| `str(x)`, `repr(x)` | `x.to_string()` or `f"{x}"`; `f"{x!r}"` |
| `int(s)`, `float(s)` | `s.parse[int]()`, `s.parse[float]()` (a `Result`) |
| `int(x)` for a float | `x as int` (rounds toward zero, and saturates) |
| `xs.append(x)` | `xs.push(x)` |
| `xs.pop()` | `xs.pop()`, which gives an `Option` |
| `xs.index(x)` | `xs.index_of(x)` (an `Option`) |
| `xs[-1]` | `xs.last()` or `xs[xs.len() - 1]` (a negative index panics) |
| `xs[a:b]` | `xs[a..b]` |
| `s.strip()`, `s.lstrip()`, `s.rstrip()` | `s.trim()`, `s.trim_start()`, `s.trim_end()` |
| `s.split()` | `s.split_whitespace()` |
| `s.splitlines()` | `s.lines()` |
| `s.startswith(p)`, `s.endswith(p)` | `s.starts_with(p)`, `s.ends_with(p)` |
| `s.upper()`, `s.lower()` | `s.to_upper()`, `s.to_lower()` |
| `s.find(t)`, -1 when absent | `s.find(t)`, an `Option` |
| `", ".join(xs)` | `xs.join(", ")` |
| `dict.get(k, d)` | `m.get_or(k, d)`; `m.get(k)` gives an `Option` |
| `True`, `False`, `None` | `true`, `false`, `None` (only as an `Option`) |
| `def f():` | `fn f():` |
| `lambda x: x + 1` | `fn(x) => x + 1` |
| `if xs:` | `if not xs.is_empty():` |
| `7 / 2` on integers | `7 // 2`, or `7 as float / 2.0` |
| `try` / `except` / `raise` | `Result`, `?`, `match`, `return Err(e)` (chapter 7) |
| `@dataclass` | `struct` (chapter 6) |

```ember
fn main():
    xs = [3, 1, 4]
    xs.push(1)
    println(xs.pop(), xs.index_of(4), xs.index_of(9), xs.last())
    s = "  Hello, World  "
    println(s.trim().to_lower(), s.trim().starts_with("Hello"))
    println([part.trim() for part in s.split(",")].join("|"))
    println("42".parse[int]().unwrap_or(0) + 1, 3.99 as int, 7 // 2)
    println(f"{xs!r}", xs.is_empty(), "x".find("y"))
```

```output
Some(1) Some(2) None Some(4)
hello, world true
Hello|World
43 3 3
[3, 1, 4] false None
```

## What is new

* **Types are checked before the program runs.** Most are worked out for you; function parameters
  and fields are written.
* **Values have one owner.** Assigning or passing on a list moves it, unless the function only
  borrows it (the default for a parameter); a moved name cannot be used again, and the error offers
  a clone (chapter 8).
* **Objects are freed when their last handle goes,** at a point you can see, not by a collector. A
  cycle of strong handles leaks, and a debug run reports it with the field to make `Weak`
  (chapter 9).
* **Integers are 64-bit, and overflow stops the program** instead of growing: `@overflow(wrap)` or
  `wrapping_add` when wrapping is wanted.
* **Threads run in parallel.** There is no global lock: the compiler refuses a data race instead.

## One program, side by side

A dice game in Python:

```python
import random

class Player:
    def __init__(self, name):
        self.name = name
        self.hp = 10
        self.inventory = []

def roll(rng):
    return rng.randint(1, 6)

rng = random.Random(42)
player = Player("Ada")
counts = {}
for turn in range(5):
    r = roll(rng)
    counts[r] = counts.get(r, 0) + 1
    if r == 6:
        player.inventory.append("gem")
    else:
        player.hp -= r // 2
print(f"{player.name}: hp={player.hp}, items={len(player.inventory)}")
for face in sorted(counts):
    print(face, counts[face])
```

The same game in Ember:

```ember
import random

class Player:
    name: String
    hp: int = 10
    inventory: Array[String] = []

fn roll(mut rng: random.Rng) -> int:
    return rng.int_in(1..7)

rng = random.Rng.seeded(42)
player = Player("Ada")
counts: Map[int, int] = {}
for turn in range(5):
    r = roll(rng)
    counts[r] = counts.get_or(r, 0) + 1
    if r == 6:
        player.inventory.push("gem")
    else:
        player.hp -= r // 2
println(f"{player.name}: hp={player.hp}, items={len(player.inventory)}")
for face in sorted(counts):
    println(face, counts[face])
```

What changed: the fields are declared, with their types, in the class body; `roll` says it changes
`rng` (`mut`); `append` is `push` and `dict.get(k, d)` is `get_or`; the generator is seeded
explicitly and draws from a half-open range; `print` with a newline is `println`. Everything else —
the script at the top level, `range`, `len`, `sorted`, iterating a map's keys, f-strings, `//` —
reads as in Python. The two programs print different numbers only because their generators differ.
