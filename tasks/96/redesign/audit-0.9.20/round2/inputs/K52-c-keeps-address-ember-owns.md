# K52-c-keeps-address-ember-owns

Question: How does code give C an object's address while Ember keeps owning it?
Severity: medium; impact rank 52
Members: J8-12; primary J8-12
Why together: Single question.
Critic's notes: The spelling follows K51. Under J2-11 B, the cast form (`body as ObjectPtr[Body]`) rather than a third `ref_to_ptr` overload.

## Finding J8-12
Rules: FFI-35a, FFI-11, FFI-23, THR-2; file tasks/spec-0.9.9/parts/p16-ffi.md lines 290, 361-365, 545-548; kind contradiction; severity medium
Quote from Hardened_11:
result-position pointer with no lifetime word. A class object given to a `retained` parameter is
  marked in its header as kept by C and never moves: handed to another thread while a `weakref` link on

The finder's question:
How does a class object come to be "given to a `retained` parameter" while Ember still owns it, and what sets
the mark in its header? FFI-11 gives a `retained` non-callback pointer no safe type (the call stays unsafe,
with a raw pointer). FFI-23 says an Ember value C keeps goes only by `owns` or `into_raw()`, and both end the
Ember name. Yet THR-2's message shows a safe `register(names)` followed by a hand-over of `names`.

Program:
```
class Names:
    items: Array[String] = []

unsafe extern "C":
    @ffi(param(p, retained, one))
    safe fn register(p: Names)     # FFI-11: no safe type, E5002?

fn main():
    names = Names()
    w = weakref names
    register(names)                # THR-2's line 4
    h = thread.spawn(fn() owns names: names.items.len())   # stops: C keeps its address
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
Verdict 114 fix 3: "an object given to C under a `retained` contract never moves; for it a link at the
hand-over stops the program", carried in FFI-35a (p16-ffi.md 363-365) and THR-2 (p11-concurrency.md 122-125,
whose message shows a safe `register(names)` on line 4). FFI-11's ownership row (290) gives `retained` on any
non-callback pointer "no safe type ... the call stays `unsafe`, with a raw pointer". FFI-23 (545-547) says an
Ember value C keeps goes only by `owns` or `into_raw()`, and both end the Ember name, so nothing on this
thread could later hand the object to another thread. `Retained.pin` (ODR-156, in the appendix's history) was
removed (verdict 91's correction). The spec defines no spelling that gives C a class object's address while
Ember keeps the owner, and THR-2's example cannot be written. Because verdict 114 decides the stop, the open
question is the spelling (which programs compile), not the behaviour.

options:
[{'letter': 'A', 'summary': "In `unsafe:`, `ref_to_ptr(body)` gives a class object's address as an
`ObjectPtr[Body]` without ending `body`. Passing it to a parameter contracted `retained` marks the object as
kept by C, so the decided stop applies.", 'code': '    # SAFETY: `world_remove` is called before `body` ends\n
unsafe:\n        world_add(ref_to_ptr(body))\n    h = thread.spawn(owns fn() owns body: body.mass)',
'outcome': "This compiles. At run time the spawn stops with THR-2's message (`w` points at it and C keeps its
address). With `w = None` first, the body goes to the thread and stays at the same address."}, {'letter': 'B',
'summary': 'There is no such way. An object C keeps goes only by `owns` or `into_raw()`, which end the name,
so it can never be handed to a thread, and the stop and the mark are removed as unreachable.', 'code': '    #
SAFETY: the world keeps the body and never frees it\n    unsafe:\n        world_add(body.into_raw())\n    h =
thread.spawn(owns fn() owns body: body.mass)', 'outcome': "The spawn line is the use-after-hand-over error
(`body` ended at `into_raw()`). The owner's decided stop never happens."}]

recommendation:
{'letter': 'A', 'reason': "It gives the decided rule a program it applies to, using the language's existing
safe address-of spelling, and keeps the unsafe call vouched for. B would quietly remove a stop the owner
chose.", 'checklist': "11 (`ref_to_ptr` and `ObjectPtr` reused), 2 (the common engine pattern compiles), 10
(keeps the owner's decided behaviour), 8"}

