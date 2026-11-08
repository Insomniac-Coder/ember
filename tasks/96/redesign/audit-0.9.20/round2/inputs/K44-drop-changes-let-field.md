# K44-drop-changes-let-field

Question: May a class's drop change what one of its let fields holds?
Severity: medium; impact rank 44
Members: J3-10; primary J3-10
Why together: Single question.
Critic's notes: 

## Finding J3-10
Rules: CLS-9a, DRP-1, CLS-7a; file tasks/spec-0.9.9/parts/p05-declarations.md lines 370-373; kind contradiction; severity medium
Quote from Hardened_11:
* `[CLS-9a]` *(changed in 0.9.15)* After `init`, nothing is changed through a `let` field, for every
  type: a number or struct field never changes, a list field is never added to, and a class-object
  field is never pointed at another object nor its object changed through it (`E2104`: "`items` is

The finder's question:
May a class's `drop` call a changing method on one of its `let` fields (flush or close it), or is that E2104?

Program:
```
from std.fs import File

class Journal:
    let file: File

    fn init(path: StrView):
        this.file = File.create(path).expect("cannot open")

    fn drop():
        this.file.flush().expect("flush failed")
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
CLS-9a (p05:370-375) says that after `init` nothing is changed through a `let` field, "for every type",
including the object a class-object field holds. CLS-7a (p05:367-368) says a `drop` "changes its object as a
given", and CLS-6 (p05:353-355) runs `drop` after the object has ended and before its fields end. A `drop`
runs after `init`, and neither rule says which wins when `drop` closes or flushes a `let` field. Verdict 69
settles only that `drop` writes no `changes this`.

options:
[{'letter': 'A', 'summary': "Inside `drop`, the object's `let` fields may be changed and ended: the object has
already ended, so no other name can see the change.", 'code': '    fn drop():\n        this.conn.close()
# allowed: the session is ending', 'outcome': 'Compiles and prints `7`, then `closed 7` when `s` ends.
Everywhere else, a change through `conn` after `init` stays E2104.'}, {'letter': 'B', 'summary': 'E2104
applies inside `drop` too: a field that `drop` changes is written without `let`.', 'code': 'class Session:\n
conn: Connection                    # `let` removed so drop may close it', 'outcome': 'The program as written
is E2104 at `this.conn.close()`, with the help to remove `let`. Then any method may change `conn`, not only
`drop`.'}]

recommendation:
{'letter': 'A', 'reason': "`let` promises that the field never changes while anyone can see the object. During
`drop` nobody can, so refusing the cleanup refuses a correct program, and B makes the field less protected for
the object's whole life.", 'checklist': "2 (accepts the correct program, refuses only what is wrong); 3
(decided when compiling); 6 (an exception only where the rule's promise is not at stake, and it says why); 7
(general for every type of `let` field)."}

