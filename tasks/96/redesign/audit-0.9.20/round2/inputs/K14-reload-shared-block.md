# K14-reload-shared-block

Question: Does a reload migrate a struct that lives in a shared block?
Severity: medium; impact rank 14
Members: J7-16; primary J7-16
Why together: Single question.
Critic's notes: Old layout under new code in reload builds (memory safety). A touches OBJ-1 and COST-3: a thread_shared block's header is 48 bytes in reload builds.

## Finding J7-16
Rules: HR-12, OBJ-1, HR-15, HR-13; file tasks/spec-0.9.9/parts/p20-annex-b.md lines 120-122; kind undecided_case; severity medium
Quote from Hardened_11:
* `[HR-12]` *(changed in 0.9.15)* In a reloadable build, class instances are registered in a live-instance
list, which adds / 16 bytes to the object header of `[OBJ-1]` (24 bytes instead of 8, and 28 instead of 12 for
an object / `weakref` links may reach); non-reloadable builds keep `[OBJ-1]`'s layout.

The finder's question:
In a reload build, are shared values of a struct (`shares mut Stats()`, `thread_shared Stats()`, an `owns`
box), which live in their own block but are not class instances, registered and migrated when `Stats`'s fields
change? And what is a `thread_shared` value's header there?

Program:
```
struct Stats:            # edit: add `misses: int = 0`
    hits: int = 0

fn main():
    s = thread_shared Stats()
    loop_forever(s)      # reload while `s` lives: migrated, refused, or left with the old layout?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
HR-12 (p20 120-122) registers only 'class instances' and gives reload header sizes for the 8- and 12-byte
headers only. Since 0.9.15, RC-1 (p08 103-104) puts any shared value, a struct included, in 'a block of its
own with the header of `[OBJ-1]`', and OBJ-1 gives a `thread_shared` value a 32-byte header that HR-12 never
mentions. HR-13 (p20 125-126) migrates value data 'through its container', but a struct in its own shared
block has no container, and HR-15 (p20 157-162) rewrites owners 'in live instances and registered containers'.
HR-13b refuses only data behind raw pointers or foreign memory. So a changed struct reached only through a
`shares` block is neither migrated nor listed as refused, against HR-1's 'all live object state preserved'.

options:
[{'letter': 'A', 'summary': 'Every counted block is registered and migrated like a class object', 'code': "#
the program above, unchanged\n# in a reload build every `shares`, `shares mut`, `thread_shared` and `owns`
block is registered as a\n# class object is, with 16 bytes more in its header (a `thread_shared` value's: 32 +
16 = 48)", 'outcome': 'The reload applies: the Stats block migrates, `hits` keeps its count and `misses`
starts at 0. Release builds are unchanged.'}, {'letter': 'B', 'summary': 'Refuse a reload that changes a type
the program may hold in such a block', 'code': '# the program above, unchanged\n# plan refuses the edit:
"`Stats` may live in a shared block (`shares mut Stats()`, line 14), which a\n#  reload cannot find; restart
to change it"', 'outcome': 'The reload is refused and the old code keeps running. Adding a field to any struct
that is ever shared needs a restart.'}]

recommendation:
{'letter': 'A', 'reason': "One model for every block that lives on its own: it keeps HR-1's promise, and costs
16 bytes per block only in reload builds, as class objects already pay. B turns an everyday edit into a
restart.", 'checklist': '2 (the edit applies), 5 (one rule for every block with an OBJ-1 header), 7 (general:
every shared kind), 13 (no unregistered side path)'}

