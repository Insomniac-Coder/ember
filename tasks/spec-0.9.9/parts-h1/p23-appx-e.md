---

# Appendix E — Coming from Python *(non-normative)*

Ember reads like Python and runs like C. Most Python habits carry over unchanged; the ones that do not
are rejected with a fix-it (`[DIA-21]`) rather than given a different meaning (`[PHIL-14]`).

## E.1 What carries over

| Python | Ember | Notes |
|---|---|---|
| indentation, `:` blocks, `#` comments | same | spaces only (`[LEX-4]`) |
| `x = 1` declares | same | a name assigned in every branch is declared after the `if` (`[CTL-10]`) |
| `a, b = b, a`; `return a, b` | same | tuples (`[GRM-29]`) |
| `[1, 2]`, `{"k": v}`, `{1, 2}` | same | `Array`, `Map` (insertion-ordered like `dict`), `Set` |
| `[x * x for x in xs if x > 0]` | same | also map and set comprehensions |
| `for x in xs:` … `else:` | same | |
| `for i, x in enumerate(xs):` | `for i, x in xs.iter().enumerate():` | |
| `for k, v in d.items():` | `for k, v in m:` (or `m.items()`) | |
| `x in xs`, `x not in xs` | same | cost reported by `ember inspect --cost` |
| `a < b < c` | same | `b` evaluated once |
| `x is None` | same | on `Option` |
| `7 // 2`, `-7 // 2 == -4`, `-7 % 2 == 1` | same | floor semantics |
| `7 / 2` on ints | `7 // 2` or `7 as float / 2` | integer `/` is rejected (`[TYP-28]`) |
| `2 ** 10` | same | overflow panics instead of growing |
| f-strings with `=`, `!r`, format specs | same | |
| `print(a, b, sep=", ", end="")` | same | |
| `input("> ")` | same | panics at end of input (`[STD-10]`) |
| keyword arguments, defaults | same | |
| `lambda x: x + 1` | `fn(x) => x + 1` | |
| generators, `yield` | `gen fn f() -> Generator[T]:` | |
| `with open(p) as f:` | `with f = fs.File.open(p)?:` | the file closes at the end of the block, or whenever it is dropped |
| `try` / `except` / `raise` | `Result`, `?`, `match`, `return Err(e)` | Part XIII |
| classes, inheritance, `super()` | `class`, `open class`, `super.init(…)` | handles are reference-counted; `Weak` for back-pointers |
| `@dataclass` | `struct` | equality, debug printing and cloning are automatic |
| `list.sort()`, `sorted(xs)` | `xs.sort()`, `xs.sorted()` | stable |
| `dict.get(k, d)` | `m.get_or(k, d)` | `m.get(k)` returns an `Option` |
| `d[k] = v`, `d[k]` | same | a missing key panics like `KeyError` |

## E.2 Names that differ (`[STD-13]`)

| Python | Ember |
|---|---|
| `len(x)` | `x.len()` (bytes for strings; `s.char_count()` for characters) |
| `str(x)`, `repr(x)` | `x.to_string()` or `f"{x}"`; `f"{x!r}"` |
| `int(s)`, `float(s)` | `s.parse[int]()`, `s.parse[float]()` (a `Result`) |
| `int(x)` for a float | `x as int` (saturates; NaN is 0) |
| `range(n)`, `range(a, b, s)` | `0..n`, `(a..b).step_by(s)` |
| `xs.append(x)`, `xs.extend(ys)` | `xs.push(x)`, `xs.extend(ys)` |
| `xs.pop()` | `xs.pop()` returns an `Option` |
| `xs.index(x)` | `xs.index_of(x)` (an `Option`) |
| `xs[-1]` | `xs.last()` or `xs[xs.len() - 1]` (negative indices panic) |
| `xs[a:b]` | `xs[a..b]` |
| `s.strip()`, `s.lstrip()`, `s.rstrip()` | `s.trim()`, `s.trim_start()`, `s.trim_end()` |
| `s.split()` | `s.split_whitespace()` |
| `s.splitlines()` | `s.lines()` |
| `s.startswith(p)`, `s.endswith(p)` | `s.starts_with(p)`, `s.ends_with(p)` |
| `s.upper()`, `s.lower()` | `s.to_upper()`, `s.to_lower()` |
| `s.find(t)` (returns -1) | `s.find(t)` (returns `Option[int]`) |
| `", ".join(xs)` | `xs.join(", ")` |
| `True`, `False`, `None` | `true`, `false`, `None` (only as an `Option`) |
| `def f():` | `fn f():` |
| `if xs:` | `if not xs.is_empty():` |
| `abs`, `min`, `max`, `round` | `abs`, `min`, `max`; `x.round()` rounds half to even, as Python's does |

## E.3 What is new

* **Types are checked before the program runs.** Most are inferred; function parameters and struct
  fields are written.
* **Values have one owner.** Assigning or passing a list moves it unless the parameter only borrows it
  (the default). A moved name cannot be used again; the error says where it moved and offers
  `.clone()`.
* **Objects are freed when their last handle goes**, at a predictable point, not by a collector. A
  cycle of strong handles leaks; the debug build reports it at exit, with the field to make `Weak`.
* **Integers are 64-bit and overflow panics.** `@overflow(wrap)` or `wrapping_add` when wrapping is
  wanted.
* **Threads run in parallel.** There is no global lock; the compiler rejects data races instead
  (Part XI).
