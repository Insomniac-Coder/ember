# K68-pack-tuple-sizes

Question: What is a tuple of a pack's types when the pack holds one type, or none?
Severity: medium; impact rank 68
Members: J2-07; primary J2-07
Why together: Single question.
Critic's notes: Rare in programs, but the library's own `View[Cs...]` and `zip` depend on it.

## Finding J2-07
Rules: TYP-47; file tasks/spec-0.9.9/parts/p04-types.md lines 495-506; kind verdict_missing; severity medium
Quote from Hardened_11:
`...` after a type parameter's name makes it a **pack**: any number of types, none
  included (`[Cs...]` above `fn view() -> View[Cs...]:`).

The finder's question:
Can a struct or class take a pack (`[Ts...]` above `struct Event:`), and what may its body write? A field
holding the pack's values, a function type `fn(Ts...)`, a tuple type `(Ts...)`, or a per-type expansion such
as `ref Pool[Cs]...`?

Program:
```
[Ts...]
class Event:
    handlers: Array[fn(Ts...)] = []          # a function type over a pack: legal?
    fn fire(values: Ts...):
        for h in this.handlers:
            h(values...)

fn main():
    hit = Event[int, String]()
    hit.handlers.push(owns fn(dmg: int, who: String): println(dmg, who))
    hit.fire(5, String("orc"))
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
Verdict 115 says 'a function or type takes any number of types' and names as uses a tuple's interfaces written
once for every size, `zip` over any number of iterators, the ECS view, and `Event[int, String]` with handlers
`fn(int, String)`; verdict 116 chose C++'s dots. TYP-47 (p04 495-510) defines a pack only on a function: its
last parameter, `Cs...` among type arguments, `values...` in a call. It does not say a struct or class may
take a pack, nor that `fn(Ts...)` or `(Ts...)` may be written, and the library's own `View[Cs...]` (p04 497,
ECS-4 p12 223-225) and `zip` (p15 34) cannot be written by its rules. Packs on types are the owner's (verdict
115); what a tuple of a pack is when the pack holds one type or none is not decided anywhere: TYP-27 (p04
58-59) says a tuple type has two or more elements, and verdict 115 began from the complaint that one type
became `(A,)`, a tuple TYP-27 refuses.

options:
[{'letter': 'A', 'summary': "A tuple of a pack holding one type is that type itself, and of none is `void`: a
tuple always has two or more elements, as written tuples do. A tuple's interfaces are written once for every
size from two up (`[A, B, Ts...]` above `extend (A, B, Ts...) implements Display:`).", 'code': 'one =
Row[int](cells=7)          # `cells` is an int\nprintln(one.cells + 1)           # 8', 'outcome': 'Compiles;
prints `ann 8` with the line above (`ann 7` as the example stands). `zip(xs)` of one list gives its items
plainly, and `reg.view[Position]()` gives each Position with no tuple around it.'}, {'letter': 'B', 'summary':
'A tuple of a pack is a tuple of any size, one and none included, distinct from the type itself and from
`void`; a one-element tuple then needs a spelling of its own, which the language does not have today.',
'code': 'one = Row[int](cells=(7,))       # a one-element tuple, a new spelling\nprintln(one.cells.0 + 1)
# 8', 'outcome': 'Compiles only once `(7,)` is added to the language; `Row[int](cells=7)` is E2020. `zip(xs)`
of one list gives one-element tuples.'}]

recommendation:
{'letter': 'A', 'reason': 'It keeps the tuple rule that exists (two or more elements), adds no spelling, and
gives exactly what the pack design was asked for: one component type with no tuple around it.', 'checklist':
'5 (a tuple means one thing everywhere), 11 (no new spelling of a tuple), 12 (no syntax missing), 13 (no
special one-element tuple only packs make)'}

