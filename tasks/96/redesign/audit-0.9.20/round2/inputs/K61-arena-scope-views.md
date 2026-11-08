# K61-arena-scope-views

Question: Does opening an arena scope end views made from the arena before it?
Severity: low; impact rank 61
Members: J5-19; primary J5-19
Why together: Single question.
Critic's notes: 

## Finding J5-19
Rules: ARN-1, ARN-6, ARN-7; file tasks/spec-0.9.9/parts/p09-memory.md lines 105-106; kind inconsistency; severity low
Quote from Hardened_11:
(`[LT-1]`, `[LT-44]`). `reset()`, `scope()` (`[ARN-6]`) and the arena's end may reuse or free that
memory: each ends every view into the arena, and a later use of one is `E3060`.

The finder's question:
Is a view allocated from the arena before `frame.scope()` still usable during and after the scope? Is `return
root.kind` below E3060?

Program:
```
@derive(Zeroable)
struct Tok:
    kind: int

struct Node:
    kind: int

fn parse(ref mut frame: Arena) -> int:
    root = ref mut frame.alloc(owns Node(kind=1))   # kept for the whole parse
    with s = frame.scope():                          # scratch memory, released last in first out
        toks = ref mut s.alloc_zeroed[Tok](64)
        toks[0].kind = 2
    return root.kind                                 # E3060 by [ARN-1], though the scope never touched root's bytes
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.5)
Reason:
The case is not missing: [ARN-1] decides it (E3060). The problem is a contradiction in the rules' own terms.
[ARN-1] (p09 102-106) says views "point into the arena's memory, not into its cursor", and that `scope()` "may
reuse or free that memory". [ARN-6] (p09 140-141) says a scope "releases everything at the block's end, last
in first out", that is, only what was taken from it. Memory taken before the scope is never reused under it,
so the rule refuses a correct program for a reason that is untrue for that program. No verdict covers scopes.
Changing this changes which programs compile, so it is the owner's; the severity is low.

options:
[{'letter': 'A', 'summary': 'A scope ends only the views made from it.', 'code': '@derive(Zeroable)\nstruct
Tok:\n    kind: int\n\nstruct Node:\n    kind: int\n\nfn parse(ref mut frame: Arena) -> int:\n    root = ref
mut frame.alloc(owns Node(kind=1))\n    with s = frame.scope():\n        toks = ref mut
s.alloc_zeroed[Tok](64)\n        toks[0].kind = 2\n        root.kind += toks[0].kind\n    return
root.kind\n\nfn main():\n    frame = Arena.with_capacity(64 * 1024)\n    println(parse(ref mut frame))',
'outcome': "Compiles and prints 3. The scope's end lowers the arena's pointer back to where the scope began,
so `root`, which lies below it, stays valid. Reading or writing `root` is not using the parent; allocating
from, resetting or scoping the parent while the scope is live stays E3096."}, {'letter': 'B', 'summary': 'As
now: opening a scope ends every view into the arena.', 'code': '@derive(Zeroable)\nstruct Tok:\n    kind:
int\n\nstruct Node:\n    kind: int\n\nfn parse(ref mut frame: Arena) -> int:\n    kind = 1
# kept in a local instead of the arena\n    with s = frame.scope():\n        toks = ref mut
s.alloc_zeroed[Tok](64)\n        toks[0].kind = 2\n        kind += toks[0].kind\n    root = ref mut
frame.alloc(owns Node(kind=kind)) # made after the scope\n    return root.kind\n\nfn main():\n    frame =
Arena.with_capacity(64 * 1024)\n    println(parse(ref mut frame))', 'outcome': 'In the example, `root.kind +=
toks[0].kind` and `return root.kind` are E3060: "`frame.scope()` (line 11) ends every view into `frame`, and
`root` (line 10) is used on line 14". This version prints 3, and the rule\'s stated reason is corrected to
"one rule for reset, scope and end".'}]

recommendation:
{'letter': 'A', 'reason': 'A scope gives back only what it took, last in first out, so memory taken before the
scope is never reused under it. The rule then refuses only what is really wrong, and the layout every parser
and frame allocator uses, a lasting tree plus scratch memory, works as written.', 'checklist': '1, 2 (accepts
the correct program), 3 (nothing runs), 7'}

