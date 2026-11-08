# K04-weakref-outlives-arena

Question: Can a weakref link to an arena-made object outlive the arena?
Severity: high; impact rank 4
Members: J5-02; primary J5-02
Why together: Single question.
Critic's notes: Use after free as written today. A extends LT-4/BCK-4/ALC-5 to weakref links.

## Finding J5-02
Rules: WK-11, OBJ-3, ALC-1, ALC-5, LT-4; file tasks/spec-0.9.9/parts/p09-memory.md lines 199-200; kind undecided_case; severity high
Quote from Hardened_11:
a value made in an instance, and every
value that holds one, lives no longer than that instance (`E3063`, `[LT-4]`)

The finder's question:
Does a `weakref` link to an object made in a memory manager's instance compile when the link outlives the
instance? If so, what does opening it do after the instance has been freed or reset?

Program:
```
class Enemy:
    hp: int

class Camera:
    target: weakref Enemy = None

fn load(ref mut cam: Camera):
    level = Arena.with_capacity(64 * 1024)
    boss = Enemy(hp=500) using level
    cam.target = weakref boss       # the link holds a weak count on boss's block, which lives in `level`
    # boss ends (block kept by the weak count), then level ends and frees its memory

fn main():
    cam = Camera()
    load(ref mut cam)
    if cam.target exists as e:      # tests a header in freed (or reused) arena memory
        println(e.hp)
```

The checker's verdict: confirmed (route owner_odr, severity high, confidence 0.75)
Reason:
No rule ties a `weakref` link to the instance its target was made in. [ALC-1], [TYP-46] and [LT-4] bind only
"a value made in an instance, and every value that holds one". [WK-11] says a link "holds one weak count", not
the value. [OBJ-3] keeps the block, which sits in the arena's memory, until the weak count reaches 0. [ALC-1]
(1) frees all of an arena's memory at once. [ALC-5] guards a reset only "while a value made in it lives".
Grepping `weakref` with arena, instance and manager finds nothing more. [PHIL-10] demands an answer (no
use-after-free), but more than one safe answer exists, so the choice is the owner's.

options:
[{'letter': 'A', 'summary': 'The link counts as holding the boss, so it lives no longer than the arena, like
the values made in it.', 'code': 'class Enemy:\n    hp: int\n\nclass Camera:\n    target: weakref Enemy =
None\n\nfn load_level(ref mut cam: Camera, ref mut level: Arena):   # the caller keeps the arena\n    boss =
Enemy(hp=500) using level\n    cam.target = weakref boss\n    println(boss.hp)\n\nfn main():\n    level =
Arena.with_capacity(64 * 1024)   # made before cam, so it ends after cam\n    cam = Camera()\n
load_level(ref mut cam, ref mut level)\n    if cam.target exists as e:               # the boss ended with
load_level; its block is still in `level`\n        println(e.hp)\n    else:\n        println("gone")',
'outcome': 'The example is E3063 at `cam.target = weakref boss`: `cam` outlives `level`, so make the arena in
the caller. This version compiles and prints 500, then gone. `level.reset()` while `cam` still holds the link
is E3021, just as a reset under a live value is.'}, {'letter': 'B', 'summary': 'A `weakref` link may only name
an object from the built-in memory manager.', 'code': 'class Enemy:\n    hp: int\n\nclass Camera:\n    target:
weakref Enemy = None\n\nfn load_level(ref mut cam: Camera):\n    boss = Enemy(hp=500)                     # no
`using`: from the built-in manager\n    cam.target = weakref boss\n    println(boss.hp)\n\nfn main():\n    cam
= Camera()\n    load_level(ref mut cam)\n    if cam.target exists as e:\n        println(e.hp)\n    else:\n
println("gone")', 'outcome': 'Every `cam.target = weakref boss` with a boss made `using` an arena is refused:
"a `weakref` link needs an object from the built-in memory manager; make the boss without `using`". This
version prints 500, then gone. A camera can never follow an arena-made enemy, even while the arena lives.'}]

recommendation:
{'letter': 'A', 'reason': 'It applies the rule the specification already has for values that hold an
arena-made value to the one holder that rule misses. It is checked when compiling, nothing runs, and the
common layout of a level arena with links into it keeps working.', 'checklist': '2 (accepts the level-arena
program, refuses only the dangling one), 3 (decided when compiling, nothing runs), 6 (the nearest rule: what
holds a value made in an instance lives no longer than the instance), 7 (every link, every manager)'}

