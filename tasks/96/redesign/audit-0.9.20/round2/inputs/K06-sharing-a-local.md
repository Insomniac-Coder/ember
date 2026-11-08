# K06-sharing-a-local

Question: What happens to a local made shared after it is made: do its views survive, and may the word be thread_shared?
Severity: high; impact rank 6
Members: J5-06, J4-05; primary J5-06
Why together: Both ask what RC-1's 'a local of this function (which becomes one of the owners)' does. J5-06 B's mechanism (the compiler places the block where the local is declared when it sees the sharing line) removes the cost J4-05 A gives as its whole reason. So one answer fixes the other.
Critic's notes: J5-06 is memory safety (a view of where the local used to be). See the recommendation change for J4-05. J7-05's lead fix is option A's edit and waits on this answer (rerouted). Verdict 23 already lets a local be shared (`other = shares mut items`), so for J4-05 the question is only whether thread_shared follows the other two sharing words.

## Finding J5-06
Rules: RC-1, HEAP-3, RC-7, BCK-4; file tasks/spec-0.9.9/parts/p08-classes.md lines 107-108; kind undecided_case; severity medium
Quote from Hardened_11:
(verdict 103): `shares x`, `shares mut x` and `thread_shared x` are written before a new value, a
local of this function (which becomes one of the owners) or a name that is already such an owner. A

The finder's question:
When a local struct value is made shared halfway through a function (it is put in a block there, `[HEAP-3]`),
does a view of it taken before that line still work after it? What is the local when only some paths share it?

Program:
```
struct Inventory:
    gold: int
    items: Array[String]

fn main():
    inv = Inventory(gold=5, items=[String("sword")])
    g = ref inv.gold                # a view of the local's own field, in the frame
    shop = shares mut inv           # [HEAP-3]: inv is put in a block of its own here
    println(g)                      # E3060, or still the gold?
    if input() == "trade":
        other = shares inv          # shared on one path only
    inv.gold += 1                   # is inv in a block here, or in the frame?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
[RC-1] lets a local be shared and stay one of the owners, and [RC-7] keeps its rights. [HEAP-3] puts the value
in a block at the line that shares it. [BCK-4] ends views only at a storage end, a hand-over, a replacement or
an end, and sharing is none of these. No rule says whether a view taken before that line survives it (grep
'becomes one of the owners', 'where it is made shared'). Whether `println(g)` compiles is the owner's
question.

options:
[{'letter': 'A', 'summary': 'Sharing a local moves it into its block at that line and ends every view into
it.', 'code': 'struct Inventory:\n    gold: int\n    items: Array[String]\n\nclass Shop:\n    stock: shares
mut Inventory\n\nfn main():\n    inv = Inventory(gold=5, items=[String("sword")])\n    shop =
Shop(stock=shares mut inv)        # inv is put in a block of its own here\n    g = ref inv.gold
# the view is taken after the move\n    println(g)\n    inv.gold += 1\n    println(shop.stock.gold)',
'outcome': 'In the example, `println(g)` is E3060: "`shares mut inv` (line 11) puts `inv` in a block of its
own, and `g` views it where it was (line 10) and is used on line 12; take `g = ref inv.gold` after line 11".
This version compiles and prints 5, then 6.'}, {'letter': 'B', 'summary': 'Views of a local survive its
sharing. Where a view crosses the sharing line, the compiler gives the local its block earlier.', 'code':
'struct Inventory:\n    gold: int\n    items: Array[String]\n\nclass Shop:\n    stock: shares mut
Inventory\n\nfn main():\n    inv = Inventory(gold=5, items=[String("sword")])\n    g = ref inv.gold\n    shop
= Shop(stock=shares mut inv)\n    println(g)\n    inv.gold += 1\n    println(shop.stock.gold)', 'outcome':
'The example compiles and prints 5, then 6. Where a view of `inv` is used across the line that shares it,
`inv` is put in its block where it is declared: the same one allocation, made earlier. Everywhere else the
block is made at the sharing line, as it is now.'}]

recommendation:
{'letter': 'B', 'reason': 'Sharing does not change which inventory `g` views, so the program means what it
says. The compiler does the placement unseen, and the cost is the same single allocation, made earlier only
where a view needs it.', 'checklist': '1, 2 (accepts a correct program), 3 (no run-time cost added), 4
(nothing written), 13 (one model: a shared local is one value for its whole life)'}

## Finding J4-05
Rules: OWN-13, RC-1, THR-1; file tasks/spec-0.9.9/parts/p07-ownership.md lines 137-142; kind contradiction; severity medium
Quote from Hardened_11:
`E2122`); `thread_shared x`, one more owner on any thread, written only
  before a new value (`names = thread_shared Names()`) or a value already `thread_shared` (`E7017`

The finder's question:
Does `shared = thread_shared names` compile, for a local `names = Names()` of this function?

Program:
```
class Names:
    lock: Lock
    items: Array[String] guarded by lock

fn share_and_keep():
    names = Names()
    shared = thread_shared names   # [RC-1]: a local of this function may be shared; [OWN-13]/[THR-1]: E7017
    println("shared")
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
A real contradiction. [RC-1] and E2122's row allow `thread_shared` before a local; [OWN-13], [THR-1] and
E7017's row refuse it. [RC-1]'s sentence came in with 0.9.15_Hardened_2's write-up of verdict 103, and that
verdict speaks only of `shares` from one-owner places. E7017 comes from the redesign's rule R3 (run 4), which
is not an owner verdict. So no owner decision covers `thread_shared` of a local.

options:
[{'letter': 'A', 'summary': 'Refused, as the rule on threads says: a value shared between threads is made that
way.', 'code': 's = thread_shared Settings(port=8080)       # made shared between threads where it is made\nh
= thread.spawn(owns fn() thread_shared s: println(s.port))\nh.join()', 'outcome': '`shared = thread_shared s`
is refused (E7017: "`s` is `main`\'s own object, which it may change; to share it between threads, create it
shared: `s = thread_shared Settings(port=8080)`"). Written as above, it compiles and prints 8080. `shares s`
and `shares mut s` of a local stay allowed.'}, {'letter': 'B', 'summary': 'Allowed, as `shares s` of a local
is: `s` becomes one of the owners.', 'code': 'shared = thread_shared s', 'outcome': 'Compiles and prints 8080.
From that line on, no owner changes the object directly (`s.port = 9` is refused). But every `Settings` object
that may be shared this way must carry the 32-byte two-count header from the moment it is made, or be moved to
a new block at that line, and no view or link of it may outlive the move.'}]

recommendation:
{'letter': 'A', 'reason': "A value shared between threads keeps its two owner counts in a 32-byte header from
the moment it is made; a one-thread object has 8 or 12 bytes. Turning an object that already exists into one
would make every object of its class pay the bigger header, or move it and cut its links. That is the owner's
own reason for refusing `shares` of a one-owner place: no cost spread to values that never need it. `shares`
of a local keeps working, because every header already holds a count.", 'checklist': "3 (no object grows), 5
(one rule for struct and class values), 7, 9 (the error's line compiles), 13 (no hidden bigger header and no
hidden move)"}

