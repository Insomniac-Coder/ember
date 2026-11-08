# K17-read-only-parameter-promise

Question: Can a function change what its read-only parameter names, through a weakref made from it?
Severity: medium; impact rank 17
Members: J5-07, J4-14; primary J5-07
Why together: Both ask whether a plain parameter's 'only reads' holds against a side route: a link made from it (J5-07) or another name of the same place (J4-14).
Critic's notes: J4-14 is rerouted to lead_fix (verdict 53), so present J5-07 alone and cite verdicts 32 and 53 as the nearest decided rules. Show the observer case (a method that only reads `this` doing `BUS.push(weakref this)`) so that the reach of A's 'follows the link through locals, fields and calls' is clear (see conflicts). Ask before K43 (J6-02).

## Finding J5-07
Rules: WK-11, WK-12, GRM-44, OWN-13, BRW-15; file tasks/spec-0.9.9/parts/p08-classes.md lines 336-338; kind inconsistency; severity medium
Quote from Hardened_11:
header that no name reads, so making or ending a link is not a change of the target: any name may
make one (`up = weakref p` from a read-only `p`; `child.parent = weakref this` in a method that
otherwise changes only `this.children`). Making, ending or opening a link touches the target's

The finder's question:
May a `weakref` link made from a read-only name be opened `exists as ref mut` and change the object, so that a
function given a plain parameter changes the caller's object? Does `inspect` below compile, and does `main`
print 10 or 0?

Program:
```
class Enemy:
    hp: int = 10

fn inspect(e: Enemy):               # a plain parameter: only reads the enemy (verdict 32, [BRW-15])
    w = weakref e                   # allowed: making a link changes nothing ([WK-11])
    if w exists as ref mut x:       # [GRM-44]: `ref mut` may change what it names
        x.hp = 0

fn main():
    boss = Enemy()
    inspect(boss)                   # no word at the call: nothing says boss changes
    println(boss.hp)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.45)
Reason:
I could not disprove it. [WK-11] lets any name make a link, "from a read-only p" included, and [GRM-44] lets a
link be opened `ref mut`. [OWN-13]'s "An owner made from another may do no more than it" covers owners, and
[FN-2a]'s E3027 covers `ref mut` before a place reached through a read-only name, but nothing covers a link
that is opened `ref mut`. Inside `inspect` no rule is broken ([BRW-1] protects `e` only while it is still
used), so today the program compiles. That runs against verdict 32: "what a function may do with a parameter
is set by the word in front of its name". The evidence that the owner sees this as a hole is indirect (verdict
32 refused the same thing done with an owner), hence the lower confidence.

options:
[{'letter': 'A', 'summary': 'A link made from a name may do no more than that name: one made from a read-only
name opens read-only.', 'code': 'class Enemy:\n    hp: int = 10\n\nfn inspect(ref mut e: Enemy):          #
says that it may change the enemy\n    w = weakref e\n    if w exists as ref mut x:\n        x.hp = 0\n\nfn
main():\n    boss = Enemy()\n    inspect(ref mut boss)              # and the call says so too\n
println(boss.hp)', 'outcome': 'In the example, `x.hp = 0` is E3023: "`x` is opened from `w`, a link made from
`e` (line 5), and `e` only reads the caller\'s enemy; write `fn inspect(ref mut e: Enemy):` and call
`inspect(ref mut boss)`". The compiler follows the link from where it was made, through locals, fields and
calls, as it follows a view. `sharedref mut e` from a read-only `e` is refused the same way (E3027, as `ref
mut e` is). A link read from a place the function may change opens as it does now. This version prints 0.'},
{'letter': 'B', 'summary': 'Allowed: a link reaches the object, not the name. The specification says so and
gives the reason.', 'code': 'class Enemy:\n    hp: int = 10\n\nfn inspect(e: Enemy):\n    w = weakref e\n
if w exists as ref mut x:\n        x.hp = 0\n\nfn main():\n    boss = Enemy()\n    inspect(boss)\n
println(boss.hp)', 'outcome': 'Compiles and prints 0. The specification states it: a plain parameter never
changes anything through its own name, but a link made from it may be opened `ref mut` like any link, so
`inspect(boss)` may change `boss`. Every view the caller holds is still checked against what `inspect`
does.'}]

recommendation:
{'letter': 'A', 'reason': 'The word in front of a parameter sets what the function may do with it, and the
language already refuses a changing owner made from a read-only name. A link made from it is the same idea and
follows the same rule.', 'checklist': '5, 6 (the nearest rule: an owner made from another may do no more than
it), 11, 15 (a plain parameter means only what it says), 16 (the verdict that refused the owner-made way
reaches the link-made way)'}

## Finding J4-14
The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
Found while disproving J4-01. A plain parameter that is not `Copy` is passed by address ([BRW-8]). [FN-1] and
[BRW-1] promise that what it reads does not change while it is used, and [BRW-21] says "the compiler holds the
callee to it". But [BRW-21]'s check and [BCK-8]'s check at the call look only at ending or moving for a plain
parameter; only a `ref mut` parameter is also checked against what the callee changes by another name. Inside
the callee, [BCK-3] does not treat the parameter and a module variable as possibly one place. So the program
compiles, and the promise breaks for a number written by the global's name. The lead's ruling of area 2 Q4
(decisions.md 86: "only 'not ended or moved' is checked ... the compiler never assumes the value unchanged")
rests on verdict 18. That verdict is about `this` in a method that changes its own object, not about plain
parameters, while verdict 5 makes a read-only name a promise that what it reads does not change while it is
used.

options:
[{'letter': 'A', 'summary': "Refused at the call, as a `ref mut` parameter's call already is: the callee may
not change, by another name, what a read-only parameter reads while it is used.", 'code': 'report(copy of
PARTY[0])           # a player of its own', 'outcome': '`report(PARTY[0])` is refused at the call ("`report`
changes `PARTY[0].hp` by the list\'s own name (its line 2) while `p` is still used (its line 3), and `p` is
`PARTY[0]`; give it a copy: `report(copy of PARTY[0])`, or in `report` use `p` before changing the list").
`report(copy of PARTY[0])` compiles and prints ana 10, and a call with a player that is not in `PARTY` is
untouched.'}, {'letter': 'B', 'summary': 'Allowed: a read-only parameter is promised only that what it reads
is not ended or moved; a number written by another name shows through it.', 'code': 'report(PARTY[0])',
'outcome': 'Compiles and prints ana 5. The rule\'s "what `a` reads does not change while `a` is used" is
narrowed to "is not ended or moved" for parameters, while inside one function `boss = enemies[0]` followed by
`enemies[0].hp = 1` stays refused.'}]

recommendation:
{'letter': 'A', 'reason': 'A read-only name then means one thing everywhere. Inside one function, `boss =
enemies[0]` followed by `enemies[0].hp = 1` while `boss` is used is already refused, and across a call a `ref
mut` parameter already gets this very check. Nothing runs while the program runs. A method that changes its
own object (the turret aimed at itself) is not a read-only name, so it is untouched.', 'checklist': '1, 2
(refuses only the call that breaks the promise), 3 (compile time only), 5, 6 (nearest rules: the read-only
promise in one function and the `ref mut` call check), 13'}

