# K35-weakref-dyn

Question: Is weakref dyn Listener, a list of observers that does not keep them alive, a type?
Severity: medium; impact rank 35
Members: J5-13; primary J5-13
Why together: Single question.
Critic's notes: UI and game observer lists. DSP-4 also omits `shares mut dyn I` (wording, the lead's).

## Finding J5-13
Rules: WK-11, DSP-4, OBJ-1, TYP-45; file tasks/spec-0.9.9/parts/p08-classes.md lines 326-328; kind undecided_case; severity medium
Quote from Hardened_11:
* `[WK-11]` *(changed in 0.9.9)* *(changed in 0.9.15)* **`weakref` links.** `weakref T` is a link that
does not keep its target alive. Its target lives on its own: a class object, or a value held `shares`
or `thread_shared`. A link may hold no target (`parent: weakref Node = None`). `weakref x` makes a link

The finder's question:
Is `weakref dyn Listener` (a list of observers that does not keep them alive) a type? Does
`bus.listeners.push(weakref hud)` convert a `weakref Hud` to it?

Program:
```
interface Listener:
    fn on_event(code: int)

class Hud:
    name: String

extend Hud implements Listener:
    fn on_event(code: int):
        println(this.name, code)

class Bus:
    listeners: Array[weakref dyn Listener] = []

fn main():
    hud = Hud(name=String("hud"))
    bus = Bus()
    bus.listeners.push(weakref hud)
    for w in bus.listeners:
        if w exists as l:
            l.on_event(1)
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
[TYP-45] (p04 413-414), IV.9, [DSP-4] and conversion 8 (p04 84-86) list and convert only `owns dyn I`, `shares
dyn I` and `shares mut dyn I`. [WK-11] lets a link name "a class object, or a value held `shares` or
`thread_shared`" without saying whether a link's type may be an interface. [OBJ-1]'s weak-count rule speaks of
class families that a link "may name". Grepping 'weakref dyn' and 'sharedref mut dyn' finds nothing, and no
rule refuses it either.

options:
[{'letter': 'A', 'summary': '`weakref dyn I` is a type: a link to any implementer that does not keep it
alive.', 'code': 'interface Listener:\n    fn on_event(code: int)\n\nclass Hud:\n    name: String\n\nextend
Hud implements Listener:\n    fn on_event(code: int):\n        println(this.name, code)\n\nclass Bus:\n
listeners: Array[weakref dyn Listener] = []\n\nfn main():\n    hud = Hud(name=String("hud"))\n    bus =
Bus()\n    bus.listeners.push(weakref hud)\n    for w in bus.listeners:\n        if w exists as l:\n
l.on_event(1)', 'outcome': 'Compiles and prints hud 1. `weakref hud` converts to `weakref dyn Listener`, as
`owns`, `shares` and `shares mut` values convert to their `dyn` forms, and the opened `l` is a read-only `dyn
Listener` view. Every class implementing `Listener` that such a link may name gets the weak count. `sharedref
mut dyn I` follows the same rule.'}, {'letter': 'B', 'summary': 'Refused: a weak link names a class, never an
interface.', 'code': 'open class Widget:\n    name: String\n\n    virtual fn on_event(code: int):\n
pass\n\nclass Hud(Widget):\n    override fn on_event(code: int):\n        println(this.name, code)\n\nclass
Bus:\n    listeners: Array[weakref Widget] = []\n\nfn main():\n    hud = Hud(name=String("hud"))\n    bus =
Bus()\n    bus.listeners.push(weakref hud)\n    for w in bus.listeners:\n        if w exists as l:\n
l.on_event(1)', 'outcome': 'In the example, `Array[weakref dyn Listener]` is refused: "a `weakref` link names
a class; link to a base class all listeners share: `weakref Widget`". This version prints hud 1, but every
listener must derive from `Widget`.'}]

recommendation:
{'letter': 'A', 'reason': 'The observer list is the commonest use of weak links in UI and game code, and every
other way of holding an interface value already has its `dyn` form. One rule covers every holding word.',
'checklist': '5, 7, 11 (spelled as the other `dyn` forms are), 12 (the case is not refused only because its
spelling was missing)'}

