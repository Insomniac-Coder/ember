# Pass 3 — Ergonomics of 0.9.9_Hardened_1

**Question:** where does a Python programmer writing ordinary code — scripts, a text adventure, a
particle system, a scene graph with parent links — meet friction that the goal ("Python-like
ergonomics") does not require, or a Python spelling whose meaning silently differs?

**Method.** The first-week program list of `[TST-8]` was walked construct by construct against H1:
script structure, printing, strings, lists, dicts, loops, classes with back-pointers, callbacks,
threads, errors and imports. Each friction point was weighed against the other two goals — a change
is proposed only where it costs no safety and no hidden run time, or names its cost. `[PHIL-14]`
(Python spelling, Python meaning) is the test for every spelling Ember shares with Python.

**Result.** 1 silent Python-meaning difference (S1), 12 frequent frictions (S2), 9 smaller ones (S3),
and 4 proposals considered and rejected, with the reason.

| Id | Sev | Friction | Proposal |
|---|---|---|---|
| E-01 | S2 | iterating one field of an object while mutating another panics: `for c in self.children: self.log.push(c.name)` — the access word is per object (§VIII.3) | per-field access state for non-`Copy` fields; the header word keeps the whole-object (`mut self`) access |
| E-02 | S1 | `for k in m:` yields `(key, value)` pairs (`[CTL-1]`); in Python it yields keys, so `for k in m: println(k)` prints tuples, silently | iterate a `Map` by keys, as Python; `m.items()` gives pairs, `m.values()` values |
| E-03 | S2 | a script needs `fn main():`; top-level statements are `E0100` (`[GRM-2]`) | the entry file may contain statements at top level, which form the body of an implicit `main`; library modules stay items-only |
| E-04 | S2 | `len(xs)`, `range(n)`, `sum(xs)`, `sorted(xs)`, `enumerate(xs)`, `zip(a, b)`, `reversed(xs)`, `any(…)`, `all(…)` are rejected with fix-its, though each has exactly Python's meaning available | accept them as prelude functions with Python's meaning; `len` of a `str` stays rejected (Python counts characters, Ember's `len` bytes) with both fix-its |
| E-05 | S2 | `sum(x * x for x in xs)` — Python's generator expression — has no Ember form | a parenthesised comprehension `(e for x in it if c)` is a lazy iterator; as the only argument of a call it needs no extra parentheses |
| E-06 | S2 | `println(p)` for a struct without `Display` is an error; Python's `print` falls back from `str` to `repr` | printing and `{x}` in an f-string use `Display` when the type has it and `Debug` otherwise, which is Python's rule |
| E-07 | S2 | a function returning `Option[T]` must write `return Some(v)`; Python returns the value or `None` | a value of type `T` converts to `Option[T]` at a coercion site (one level only, never `Option[Option[T]]`) |
| E-08 | S2 | a function returning `Result[void, E]` must end with `return Ok(())` | falling off the end of such a function returns `Ok(())` |
| E-09 | S2 | `thread.spawn(fn() => work(x))` is rejected until the lambda is written `owned fn` | a lambda written directly as the argument of an `owned fn` parameter captures by move; `owned` is needed only when the lambda is stored first |
| E-10 | S2 | `import math`, `import random`, `import time` fail with `E1060`; Python users expect standard modules at top level | a standard module may be imported without `std.` (`import math` is `std.math`) unless a module of the package has that name, which wins with warning `W1003` |
| E-11 | S2 | `for i, x in xs.iter().enumerate():` is long | adapters are available directly on anything `Iterable` (`xs.enumerate()`, `xs.map(…)`), borrowing it as `iter()` would |
| E-12 | S2 | a float prints however the implementation likes; Python prints the shortest text that reads back (`0.30000000000000004`, `1.0`) | `Display` of a float is the shortest round-trip form with `.0` for integral values, as Python's `repr` |
| E-13 | S2 | a field assigned in `init` but not declared gets a generic "unknown name" error | shape N13: "field `x` is not declared" with a machine-applicable fix-it adding `x: <inferred type>` to the class body |
| E-14 | S3 | Python `match … case p:` fails with a parse error | `[DIA-21]` row: `case p:` → `p:` |
| E-15 | S3 | `global x`, `nonlocal x`, `*args`, `**kwargs`, `def f(self)` get parse errors | `[DIA-21]` rows: statics or a class; closures capture automatically; no variadic functions (non-goal); `fn` |
| E-16 | S3 | `print(f"{x:.2f}")` works, but `print("%d" % n)` and `"{}".format(n)` fail with type errors | `[DIA-21]` rows pointing at f-strings |
| E-17 | S3 | `x = None` then `x = 5` later: `None` alone cannot be typed | `[TYP-23]` infers `Option[T]` from later assignments of `T` (with E-07) |
| E-18 | S3 | `while True:` → `true` fix-it exists; `if x == None` works only as a fix-it | keep; both already diagnosed |
| E-19 | S3 | integer and float mix (`i * 0.5` with `i: int`) is `E2020` | keep (no implicit conversion is a non-goal), but N4's help writes `i as float * 0.5` as a fix-it |
| E-20 | S3 | the error for a moved list (`b = a; a.push(1)`) offers `clone` | keep, and add a note that Ember lists move where Python's alias, linking Appendix E |
| E-21 | S3 | `ember run` output for a panic shows a Rust-like message | keep the format; add the Python-style "Traceback"-like call list in `debug` (already `[RT-4]`); no change |
| E-22 | S3 | Appendix E does not show a whole first program side by side | add a Python/Ember side-by-side of one 30-line program |

## Considered and rejected

* **Negative indices from the end (`xs[-1]`).** Python meaning, but a mistaken `-1` from an off-by-one
  would silently read the last element instead of panicking, in a language used for engine code; and
  every index not provably non-negative would pay a branch. H1's rule stays: a literal `-1` is `E2011`
  with the `xs.last()` fix-it; a negative value panics.
* **`with open(p) as f:`.** `e as f` is already a cast, so `with e as x` would need a special case in the
  grammar; the fix-it (`with f = …:`) is enough.
* **Implicit `str` → `String` for values.** It would hide an allocation (`[PHIL-15]`); literals already
  convert (`[TXT-9]`), and `.to_string()` is a fix-it.
* **Declaring fields by assigning them in `init`.** A misspelled `self.nmae = x` would silently create a
  field; the N13 fix-it (E-13) gives the convenience without the trap.
