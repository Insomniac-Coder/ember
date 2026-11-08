# K28-untyped-list-of-text

Question: With no type written, is ["ann", "bob"] a list of Strings, as {"ann", "bob"} is?
Severity: medium; impact rank 28
Members: J3-13; primary J3-13
Why together: Single question.
Critic's notes: The draft must say that A makes untyped text lists allocate, which `@noalloc` (E4001) and PHIL-2's list of allocating forms must reflect. The speed cost against C is one allocation per element.

## Finding J3-13
Rules: TYP-38, LEX-20, TXT-9, TYP-23; file tasks/spec-0.9.9/parts/p06-expressions.md lines 91-99; kind inconsistency; severity medium
Quote from Hardened_11:
    allocation) or `Span[T]` (a view of a temporary fixed array). With no context it is `Array[T]`.
    `dict` does; `{1, 1}` is `{1}`. With no context a text key, value or element is a `String`, since
    a map written with no type owns its text; it holds `StrView` only where the program says so

The finder's question:
With no context, is `names = ["ann", "bob"]` an `Array[StrView]` (so `names.push(owns input())` is E2020),
while `seen = {"ann", "bob"}` is a `Set[String]`?

Program:
```
fn main():
    names = ["ann", "bob"]
    seen = {"ann", "bob"}
    newcomer = input()
    seen.insert(owns copy of newcomer)
    names.push(owns newcomer)
    println(names, seen)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.5)
Reason:
TYP-38 (p06:96-99) makes text in a map or set literal with no context a `String`, "since a map written with no
type owns its text". For a list, "With no context it is `Array[T]`" (p06:92), and T is the literal's own type,
`StrView` (LEX-20, p02:161-162). TYP-23 (p04:832-837) fixes from later uses only a type left open (`[]`,
`None`), so nothing later turns the list into `Array[String]`. The program is decided (the `push` is refused),
but the same reason holds for a list and no reason is given for the difference. STD-11 (p15:213-214) shows
that maps may hold views too, so the gap is not explained by maps being unable to.

options:
[{'letter': 'A', 'summary': 'One rule for every collection literal: with no type written, text in a list is
`String` too.', 'code': '    names = ["ann", "bob"]              # Array[String]\n    names.push(owns
newcomer)', 'outcome': 'Compiles. For the input `cid` it prints `["ann", "bob", "cid"]` and the set with `cid`
added. Each literal element allocates a `String`, as in a map or set today, and a list of static views is
written `labels: Array[StrView] = ["a", "b"]`.'}, {'letter': 'B', 'summary': "A list keeps its literal's own
type, `Array[StrView]`, as `total = 0` keeps `int`.", 'code': '    names: Array[String] = ["ann", "bob"]   #
needed to add a String later\n    names.push(owns newcomer)', 'outcome': 'The program as written is E2020 at
`names.push(owns newcomer)`, with the help to write the type on `names`. A list of literals allocates no
`String`s.'}]

recommendation:
{'letter': 'A', 'reason': "The spec's own reason for maps and sets, that a collection written with no type
owns its text, holds for a list too. The Python-style program then works. A program that wants zero-cost views
says so with the type.", 'checklist': '2 (accepts the ordinary program); 5 (one rule for list, map and set
literals); 6 (no exception for lists without a reason); 10 (reads and works as in Python).'}

