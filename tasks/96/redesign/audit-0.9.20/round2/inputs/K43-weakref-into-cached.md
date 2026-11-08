# K43-weakref-into-cached

Question: May a weakref link point into what a @cached method reads, and change it?
Severity: medium; impact rank 43
Members: J6-02; primary J6-02
Why together: Single question.
Critic's notes: Ask after K17. A's reason against B ('needs a second way of tracking links') no longer holds if J5-07 A brings in exactly that tracking (see conflicts).

## Finding J6-02
Rules: EFF-27, CLS-12, WK-11, WK-12; file tasks/spec-0.9.9/parts/p10-effects.md lines 158-167; kind undecided_case; severity medium
Quote from Hardened_11:
that way, so a `sharedref mut` link, or a `shares mut` owner, made into anything a `@cached` method of
  its holder reads is `E2117` at the line that makes it

The finder's question:
Does a `weakref` link to a class object that a `@cached` method of its holder reads compile? If it does, what
does the second `first_name()` print after the object is changed through the opened link?

Program:
```
class Book:
    name: String

class Shelf:
    items: Array[Book] = []

    @cached
    fn first_name() -> String:
        return copy of this.items[0].name

fn main():
    shelf = Shelf()
    shelf.items.push(owns Book(name="Dune"))
    w = weakref shelf.items[0]
    println(shelf.first_name())
    if w exists as ref mut b:
        b.name = String("Emma")
    println(shelf.first_name())   # "Dune" (stale) or "Emma"?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
EFF-27 (p10 158-166) explains that a change not made through the method's object cannot be cleared, then
refuses only `sharedref mut` links and `shares mut` owners made into what a @cached method reads. A `weakref`
link to an element living in its own block is allowed (WK-11, p08 326-339; WK-16 refuses only parts), and it
can be opened to change the element (`if ui exists as ref mut me:`, GRM-44 spec 838; `if this.target exists as
ref mut t: t.health -= 1`, EXC-15 spec 4195). CLS-12 (p05 386) covers only a weakref field the method itself
reads. So the kept result goes stale with no rule refusing it. No verdict covers it.

options:
[{'letter': 'A', 'summary': 'Refused where the link is made, as a `sharedref mut` link is', 'code': 'w =
weakref shelf.books[0]', 'outcome': 'E2117 at that line: "`shelf.books[0]` is read by `Shelf.total_pages`,
which is `@cached` and would not see a change made through this link; link the shelf instead, `weakref shelf`,
or remove `@cached`". `weakref shelf` compiles, and a change through it clears the kept result.'}, {'letter':
'B', 'summary': 'The link is allowed; a change through it is refused where it is made', 'code': 'w = weakref
shelf.books[0]\nif w exists as ref mut b:\n    b.pages = 600', 'outcome': 'The link compiles and may be opened
to read; `b.pages = 600` is E2117 (the compiler follows where the link was made, as it follows functions put
into fields). A link that is only read costs nothing and is never refused.'}]

recommendation:
{'letter': 'A', 'reason': "The rule's own reason (a change that does not pass through the holder cannot clear
the result) covers every link that can change the object; A gives the `weakref` link the same line, code and
help as `sharedref mut`, decided when compiling. B accepts more programs but needs a second way of tracking
links for this one rule.", 'checklist': '2, 3, 5, 6 (same as the sharedref mut rule beside it), 9 (help gives
`weakref shelf`), 11'}

