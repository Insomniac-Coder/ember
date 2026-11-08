# K18-string-plus

Question: Does String + read its left side and make a new String, or hand it over with a word?
Severity: high; impact rank 18
Members: J2-04, J6-14; primary J2-04
Why together: The same question, found twice. J6-14 says the 'outside Add' half is settled by TYP-21's exception, and J2-04 asks to remove that exception. Both recommend A.
Critic's notes: Very common code. Use J6-14's short program to open and J2-04's generic `with_suffix` line to show the Add bound. One B spelling: `(owns label) + "!"` (by verdict 161 it is the same as `owns label + "!"`, but it reads unambiguously). See the recommendation change: A should reuse a temporary left operand.

## Finding J2-04
Rules: TYP-21, TYP-40, TXT-11; file tasks/spec-0.9.9/parts/p04-types.md lines 745-746, 751-752; kind contradiction; severity medium
Quote from Hardened_11:
A type has an operator only by implementing its interface (`[TYP-40]`); a method that merely shares the name
is not one. ... `bool`, `char` and `String` implement none of them (`String`'s `+` takes its left operand
over, `[TXT-11]`).

The finder's question:
Is `String`'s `+` (and `s += t`) an `Add`/`AddAssign` implementation or not? Does a generic function bounded
by `Add` accept a `String`?

Program:
```
[T: needs Add[StrView, Output = T]]
fn suffix(owns a: T, b: StrView) -> T:
    return a + b

fn main():
    s = String("hp: ")
    s += "10"                    # TXT-11: appends
    t = s + "!"                 # TXT-11: takes `s` over
    u = suffix(owns t, "?")     # TYP-21: String implements no Add, so E2040?
    println(u)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.8)
Reason:
TYP-21 (p04 745-746) says a type has an operator only by implementing its interface, and (p04 751-752) that
String implements none of the operator interfaces, while TXT-11 (p15 163-164) gives String a `+` that 'takes
`a` over' and a `+=`. So String's `+` is a path no interface declares, a String fails an `Add` bound while `a
+ b` on it compiles, and TYP-21's own desugaring (p04 741-742: 'both operands read in place') is broken by a
`+` that takes its left side over with no `owns` written, which verdicts 84, 145 and 147 require of every
hand-over. No verdict decides String's `+`.

options:
[{'letter': 'A', 'summary': "String implements `Add[StrView, Output = String]`, reading both sides and giving
a new String (as Python and C++'s `std::string` `+` do), and `AddAssign[StrView]`, appending in place. Its `+`
goes through the interfaces like every other type's.", 'code': 'shown = label + "!"          # a new String;
`label` stays\nlabel += "10"                # appends in place, no new String', 'outcome': "Compiles; prints
`hp: 10`, then `hp: 10!?`. `with_suffix` accepts a String. Each `+` makes one new String (one allocation and
copy); `+=` in a loop costs what C++'s `append` costs."}, {'letter': 'B', 'summary': "String implements `Add`
with its `add` taking its object over (Rust's way, no copy), and the line says so with the word every
hand-over uses.", 'code': 'shown = (owns label) + "!"   # `label` handed over and extended in place\nlabel =
String("hp: ")       # `label` must be given a value again before use', 'outcome': '`shown = label + "!"`
without `owns` is refused, its help giving `(owns label) + "!"`; `println(label)` after it is E3040. No copy
is made. `with_suffix` must take `owns a: T`.'}, {'letter': 'C', 'summary': "Keep String's `+` built in,
outside the operator interfaces, taking its left side over with no word.", 'code': 'shown = label + "!"
# `label` silently handed over', 'outcome': 'As today: `println(label)` after it is E3040 and
`with_suffix(shown, "?")` is E2040.'}]

recommendation:
{'letter': 'A', 'reason': "It removes the only operator that bypasses its interface and the only hand-over not
written with `owns`, and matches TYP-21's own 'both operands read in place'. `+=` keeps the fast in-place
path; B is faster for long `+` chains but makes every label line read `(owns label) + ...`.", 'checklist': "2
(correct generic code accepted), 4 (nothing handed over without a word), 5 (one operator rule for every type),
10 (Python's and C++'s meaning), 13 (no hidden built-in path)"}

## Finding J6-14
Rules: TXT-11, OWN-9, FN-2a; file tasks/spec-0.9.9/parts/p15-stdlib.md lines 163-164; kind inconsistency; severity medium
Quote from Hardened_11:
`as_str` (a `StrView` of the whole text). `a + b` for `a: String, b: StrView` takes `a` over and
  returns it extended; `s += t` appends.

The finder's question:
After `full = first + " Lovelace"`, is `println(first)` E3040? Does `return name + "!"` compile when `name` is
a plain (read-only) `String` parameter?

Program:
```
fn greet(name: String) -> String:
    return name + "!"          # takes over a parameter that only reads?

fn main():
    first = String("Ada")
    full = first + " Lovelace"
    println(full)
    println(first)              # E3040 with no word on line 6 saying first was handed over?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
TYP-21 (p04 741-742; spec 1841-1842) settles that String's `+` is not `Add` ("`String`'s `+` takes its left
operand over, `[TXT-11]`"), so the 'outside Add' part is refuted. Still open: TXT-11 (p15 163-164) hands the
left operand over with no word on the line, while every other hand-over is written with `owns` (FN-2a, the
owner's rule that a word in the line says what each name is), and every other operator reads both operands in
place (TYP-21). So `println(first)` after `first + " Lovelace"` is E3040 with no word to point at, and `return
name + "!"` on a read-only parameter is refused with no stated reason. No verdict speaks of operators.

options:
[{'letter': 'A', 'summary': '`+` reads both sides and makes a new String', 'code': 'full = first + "
Lovelace"\nreturn name + "!"', 'outcome': 'Both compile; prints "Ada Lovelace", "Ada", "Ada Lovelace!". One
allocation, as an f-string makes; adding in place stays `s += t`. String then implements `Add` like other
types, and the exception in the operator rule goes.'}, {'letter': 'B', 'summary': '`+` takes its left side
over, written with `owns`', 'code': 'full = owns first + " Lovelace"', 'outcome': 'Compiles and reuses
`first`\'s buffer; `first + " Lovelace"` without `owns` is an error giving this line; `println(first)` after
it is E3040; `greet` must take `owns name: String` and be called `greet(owns full)`.'}, {'letter': 'C',
'summary': 'As today: taken over with no word', 'code': 'full = first + " Lovelace"', 'outcome': 'Compiles;
`println(first)` is E3040; `return name + "!"` is refused because a read-only parameter cannot be handed
over.'}]

recommendation:
{'letter': 'A', 'reason': "A makes `+` mean the same for every type (operands read in place), needs no hidden
hand-over, and costs what C++'s `a + b` on two named strings costs; the in-place append is already spelled `s
+= t`.", 'checklist': '4 (no hidden hand-over), 5, 6 (the operator rule without an exception), 7, 11, 13'}

