# K30-shared-owner-equality-and-identity

Question: On shared owners and links, does == compare contents and is compare identity?
Severity: medium; impact rank 30
Members: J2-09, J3-12; primary J2-09
Why together: J2-09 A gives == on owners the contents and relies on 'is still asks whether two owners own the same value', but EXP-9 refuses `is` on anything but class objects and views until J3-12 A widens it. One answer fixes the other.
Critic's notes: Add J3-12's two weakref links (`cam.target is hud.target`) as the second line.

## Finding J2-09
Rules: TYP-14, TYP-5, TYP-36, TYP-39, IV.11; file tasks/spec-0.9.9/parts/p04-types.md lines 82-83, 286-287, 688-692; kind undecided_case; severity medium
Quote from Hardened_11:
A reference, and an `owns T`, is read through wherever a `T` is wanted —
  a field access, a method call, an operand, an argument to a plain parameter.

The finder's question:
Is a `shares T`, `shares mut T` or `thread_shared T` value read through as a `T` for an argument to a plain
parameter or an operand? Does it implement `Display`, `Eq` and `Hash` as `T` does (`println(other)`, `a ==
b`)?

Program:
```
fn total(xs: Span[int]) -> int:
    return sum(xs)

fn main():
    items = [1, 2, 3]
    other = shares mut items
    other.push(owns 4)          # IV.11 strips `shares mut` for a method call
    println(total(other))       # read through to Array[int], then to Span[int]?
    println(other)              # `shares mut Array[int]` Printable?
    println(other == [1, 2, 3, 4])
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.6)
Reason:
TYP-14 (p04 286-287) and TYP-5 rule 7 name only references and `owns T` as read through where a T is wanted.
IV.11 (p04 854-855) strips one level of `shares`/`shares mut`/`thread_shared` for method calls only, IV.6a
reads a field through one (`reader.hp`, p04 386), and RC-2a/RC-2b (p08 136-139) speak of passing a shared
value to a plain parameter as a view, which assumes the read-through without stating it. TYP-36 (p04 688-692)
and TYP-39 (p04 727-728) give `ref T` and `owns T` the interfaces of T; nothing gives them to shared owners,
so `println(scores)`, `a == b` and a shared map key are undecided. Rust's `Rc` compares contents while C++'s
`shared_ptr` compares addresses, so `==` has a real choice.

options:
[{'letter': 'A', 'summary': "A `shares`, `shares mut` or `thread_shared` owner reads as the value it owns
wherever that value is wanted, as an `owns T` does: an argument, an operand, and `Eq`, `Ord`, `Hash`,
`Display` and `Debug` as the value's type has them. `is` still asks whether two owners own the same value; a
class object's `==` stays identity, its own.", 'code':
'println(total(scores))\nprintln(scores)\nprintln(scores == others)\nprintln(scores is others)', 'outcome':
'Compiles; prints `60`, `[10, 20, 30]`, `true`, `false`.'}, {'letter': 'B', 'summary': "Read through for
arguments and printing as in A, but `==` and hashing of two owners compare which value they own, as C++'s
`shared_ptr` does.", 'code': 'println(scores == others)', 'outcome': 'Compiles; prints `60`, `[10, 20, 30]`,
`false`. To compare the lists the programmer writes a function with two plain parameters; `==` and `is` then
mean the same thing on owners.'}]

recommendation:
{'letter': 'A', 'reason': "The owner word says who owns the value, not what it is, so the value behaves the
same under every owner word; `is` already exists for 'the same value', so `==` keeps one meaning.",
'checklist': "5 (one meaning of == and of each owner word), 6 (the nearest rule, owns T's), 11 (`is` already
spells identity), 13 (no owner word left on a side path)"}

## Finding J3-12
Rules: EXP-9; file tasks/spec-0.9.9/parts/p06-expressions.md lines 74-81; kind contradiction; severity low
Quote from Hardened_11:
* `[EXP-9]` *(new in 0.9.9)* *(changed in 0.9.9)* *(changed in 0.9.15)* `a is b` tests whether two class
  objects (or two views) are the same object, by comparing addresses.

The finder's question:
Does `a is b` compile for two `shares` owners of one list, or for two `weakref` links ("do the camera and the
HUD target the same enemy?")?

Program:
```
class Enemy:
    hp: int = 10

struct Hud:
    target: weakref Enemy = None

fn same_list(a: shares Array[int], b: shares Array[int]) -> bool:
    return a is b

fn same_target(cam: Hud, hud: Hud) -> bool:
    return cam.target is hud.target
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.6)
Reason:
EXP-9 (p06:74-81) lets `a is b` compare "two class objects (or two views)" and makes "Any other operand of
`is`" E2150. The operator table (p06:60) says "class objects and views". The diagnostics table's E2150 row
(p17a:96) refuses only operands "neither class objects, links, references nor `Option`s", so it admits links
(`weakref`, `sharedref mut`). TYP-45 (p04:405-412) counts `weakref` as a link, not a view, and a `shares`
owner of a list is not a class object. The rule and the table decide `cam.target is hud.target` in opposite
ways.

options:
[{'letter': 'A', 'summary': '`is` compares what two links or two owners point at: `weakref`, `sharedref mut`,
`shares` and `thread_shared`.', 'code': '    return cam.target is hud.target            # one address
compare', 'outcome': "Compiles and prints `true`. Two links that hold nothing are `is` each other. A link
whose enemy has ended still has that enemy's block, which stays while any `weakref` points at it, so the
compare stays exact. `a is b` for two `shares` owners of one list works too."}, {'letter': 'B', 'summary':
'`is` stays for class objects and views; a link is opened first.', 'code': 'fn same_target(cam: Hud, hud: Hud)
-> bool:\n    if cam.target exists as a:\n        if hud.target exists as b:\n            return a is b\n
return false', 'outcome': 'The program as written is E2150, and the error table is corrected to match. The
opened form above compiles, but it answers `false` once the enemy has ended, even though both links name the
same one.'}]

recommendation:
{'letter': 'A', 'reason': '"The same object" means the same thing for an object reached directly, through a
view or through a link. Each case is one address compare, safe because a `weakref`\'s target block stays in
place.', 'checklist': '2 (accepts the correct program); 3 (one compare, decided when compiling); 5 (one
meaning of `is` for every way of reaching an object, and the table and the rule agree); 7 (general).'}

