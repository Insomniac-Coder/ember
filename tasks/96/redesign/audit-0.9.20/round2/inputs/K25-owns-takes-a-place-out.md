# K25-owns-takes-a-place-out

Question: Does owns before a part or a downcast hand it over (body = owns req.body, owns cast s as Circle)?
Severity: medium; impact rank 25
Members: J4-04, J2-10, J5-12; primary J4-04
Why together: One rule answers all three: 'owns before a place that = would only view hands that place's value over', with a cast of a place being a place (TYP-44). Return of such a place hands it over with no word (verdict 148, OWN-11). If J4-04 is B, the downcast move is left with no spelling.
Critic's notes: J4-04 is the plainest case (Rust's `let body = req.body`). Add J2-10's `circles.push(owns cast s as Circle)` and J5-12's `return cast u as Player` as the second and third lines.

## Finding J4-04
Rules: BRW-12, OWN-11, GRM-15, OWN-10; file tasks/spec-0.9.9/parts/p07-ownership.md lines 117-119, 196-207; kind undecided_case; severity medium
Quote from Hardened_11:
`word = line.to_lower().trim()` or `best = a.as_span()`, is a read-only view of it: never a hand-over,

The finder's question:
Does `body = owns req.body` compile, handing the field out into a local? Or is there no way to move a part
into a local with `=`?

Program:
```
struct Request:
    path: String
    body: String

fn take_body(owns req: Request) -> String:
    body = owns req.body       # hands the part over ([OWN-11]) or not a place `owns` is written ([GRM-15])?
    println(req.path)          # `req.path` stays usable ([OWN-11])
    return body
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
Not settled. [BRW-12] makes `body = req.body` a view, and its list of words that say otherwise has no `owns`.
[GRM-15] lists where `owns` is written. That list includes `b = owns n` for a value `=` would copy, a use that
is optional and changes the meaning, but not `b = owns p.part`. E0109 does not cover it either, since `=` with
a part does not hand anything over. [OWN-11] and [EXP-6] let a part be handed over on its own, but name only
push, return, a field and an `owns` argument. Verdict 78 decided a field hand-over "as in Rust" on a `push`,
not on `=`. No spec example writes `x = owns p.f`.

options:
[{'letter': 'A', 'summary': '`owns` before a part on an `=` line hands the part over into the local.', 'code':
'body = owns req.body\nprintln(req.path)\nreturn body', 'outcome': 'Compiles; prints /upload, then hello.
Until `req.body` is given a new value, `req.body` and `req` used whole are refused (E3040), as after
`names.push(owns req.body)`. A field of a type with a `drop`, or of a class object, is refused as for any part
(E3010, E3013).'}, {'letter': 'B', 'summary': 'Refused: a part goes into a local only as a view or a copy;
taking it out is `mem.take`.', 'code': 'body = mem.take(ref mut req.body)   # leaves an empty String in
req.body', 'outcome': '`body = owns req.body` is refused ("`=` with a part makes a view; to take the body out
write `body = mem.take(ref mut req.body)`"). The `mem.take` line compiles and prints the same, and leaves an
empty string in the request.'}]

recommendation:
{'letter': 'A', 'reason': 'The owner already decided that a field may be handed over on its own, as in Rust.
`owns` before a value on an `=` line already hands over a copied value (`f = owns flag`), so the same word
hands a part over into a local, with no default value written into a request that is about to end.',
'checklist': "2, 5 (one meaning of `owns` on every line), 11 (the language's own word, written before a value
as it already is), 12 (adds the missing place instead of sending programs to `mem.take`)"}

## Finding J2-10
Rules: TYP-44, DSP-6; file tasks/spec-0.9.9/parts/p04-types.md lines 808-812; kind undecided_case; severity medium
Quote from Hardened_11:
`cast s as Circle`
  gives the `Circle`, on `owns dyn` and `shares dyn` values too, ... A cast of a place is a place: the name
bound from it is read-only
  unless written `ref mut`

The finder's question:
How does a program take an `owns Circle` out of an `owns dyn Shape` (Rust's `Box::downcast`), or make one more
`shares Circle` owner from a `shares dyn Shape`? What is the type of `cast` applied to a temporary?

Program:
```
fn keep_circles(owns shapes: Array[owns dyn Shape]) -> Array[owns Circle]:
    circles: Array[owns Circle] = []
    for s in owns shapes:
        if s is Circle:
            c = cast s as Circle          # a read-only view of `s` (a place), or `s` handed over?
            circles.push(owns c)          # allowed? E3010?
    return circles
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.65)
Reason:
TYP-44 (p04 808-813) and DSP-6 (p08 306-312) give `cast s as Circle` on `owns dyn` and `shares dyn` values,
and say a cast of a place is a place bound read-only or `ref mut`, never handed over. OWN-13 (p07 135-136)
says a word before a value acts on the object it names, and the grammar (p03 316) puts `owns` and `cast … as`
at one level, so `owns cast s as Circle` parses, but no rule says what it gives: the same block as an `owns
Circle` (Rust's `Box::downcast`), a refusal, or something else. A cast of a temporary is not covered either.
No verdict (25 and 27 decided the spelling only).

options:
[{'letter': 'A', 'summary': 'The words before a value act on a cast as on any value: `owns cast s as Circle`
takes `s` over and gives its block as an `owns Circle`, with no copy and no new block; `shares cast s as
Circle` on a `shares dyn Shape` is one more owner, typed `shares Circle`. A cast of a temporary gives the
value, held as the temporary was.', 'code': 'circles.push(owns cast s as Circle)', 'outcome': 'Compiles;
prints `1`. A wrong kind stops the program as any cast does (inside `if s is Circle:` it cannot).'},
{'letter': 'B', 'summary': 'A cast only gives a view; to keep the Circle the program copies it.', 'code':
'circles.push(owns copy of cast s as Circle)   # Circle needs Clone', 'outcome': '`owns cast s as Circle` is
refused ("a cast gives a view of `s`\'s Circle; to keep one write `owns copy of cast s as Circle`"); with the
copy it compiles and prints `1`, at one new block and one copy per circle, and `s`\'s block is freed.'}]

recommendation:
{'letter': 'A', 'reason': 'The words before a value already mean the same before every value, a cast included,
and A costs nothing where C++ and Rust cost nothing; B forces a copy only because the line has no meaning for
the word.', 'checklist': '5 (each word one meaning before every value), 7 (no ceiling: no copy forced), 11
(reuses `owns`/`shares` before a value), 12 (no case exempted for missing syntax)'}

## Finding J5-12
Rules: DSP-6, DSP-4, TYP-44; file tasks/spec-0.9.9/parts/p08-classes.md lines 306-309; kind undecided_case; severity medium
Quote from Hardened_11:
* `[DSP-6]` *(new in 0.9.15)* **Casts.** `cast s as Circle` gives the `Circle` and stops the program if
`s` is not one, in every profile: "`cast s as Circle` on line 9: `s` is a `Square`, not a `Circle`".
A cast of a place is a place: the name bound from it is read-only unless written `ref mut`

The finder's question:
How does a program take over an owner as its concrete kind (base-class owner to derived owner, `owns dyn I` to
`owns T`)? Does `return cast u as Player` below hand the player over, or return a view of the parameter
(E3063)?

Program:
```
open class Unit:
    hp: int

class Player(Unit):
    score: int = 0

fn promote(owns u: Unit) -> Player:
    return cast u as Player            # the Player handed over, or a view of the local `u`?

fn take_circle(owns s: owns dyn Shape) -> Circle:
    return cast s as Circle            # the Circle moved out of its block?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
[TYP-44] (p04 809-812) and [DSP-6] say a cast "gives the `Circle`, on `owns dyn` and `shares dyn` values too",
and that "A cast of a place is a place: the name bound from it is read-only". [FN-2a] (p05 131-132) has
`return` hand a value over "as `=` does", but `=` from a cast binds a view. So `return cast u as Player` is
either a hand-over or a returned view of a local (E3063). No spelling takes over an owner as its kind, and
`shares` before a cast is not among [RC-1]'s forms either. Verdicts 25 and 27 decided only the spelling of the
kind test and of the sure cast.

options:
[{'letter': 'A', 'summary': 'A cast of a whole owner is that owner, as its kind: `return cast u as Player` and
`p = owns cast u as Player` hand it over.', 'code': 'open class Unit:\n    name: String\n\nclass
Player(Unit):\n    score: int = 0\n\nfn take_player(owns u: Unit) -> Player:\n    return cast u as Player
# hands the player over; stops if `u` is not one\n\nfn main():\n    u: Unit = Player(name=String("ann"))\n
p = take_player(owns u)\n    p.score += 10\n    println(p.name, p.score)', 'outcome': 'Compiles and prints ann
10. If `u` is not a `Player`, the program stops at the cast, before anything is handed over. In a function
body, `p = owns cast u as Player` hands it over the same way, while a plain `p = cast u as Player` stays a
read-only view. The same holds from `owns dyn Shape` to `Circle`, and `shares cast s as Circle` makes another
owner.'}, {'letter': 'B', 'summary': 'A cast gives only a view, and the owner stays typed as its base class.',
'code': 'open class Unit:\n    name: String\n\nclass Player(Unit):\n    score: int = 0\n\nfn take_player(owns
u: Unit) -> Unit:    # keeps the base type\n    if u is Player as ref mut p:\n        p.score += 10\n
return u\n\nfn main():\n    u: Unit = Player(name=String("ann"))\n    v = take_player(owns u)\n    if v is
Player as p:\n        println(p.name, p.score)', 'outcome': 'In the example, `return cast u as Player` is
E3063: it returns a view of `u`, which ends when `take_player` returns. This version prints ann 10, and every
use of the player as a `Player` tests its kind again.'}]

recommendation:
{'letter': 'A', 'reason': 'It reuses what the language already spells: `return` hands a value over with no
word, and `owns` before a value hands it over in an assignment (`f = owns flag`). Nothing new is added, and a
common engine pattern, taking a component or a pool out as its concrete type, gets a direct spelling.',
'checklist': '2 (accepts the program it means), 7, 11 (the hand-over is spelled as it is everywhere else)'}

