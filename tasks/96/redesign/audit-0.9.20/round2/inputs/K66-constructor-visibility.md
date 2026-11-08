# K66-constructor-visibility

Question: May another module of the package build a struct whose fields are pub(package)?
Severity: low; impact rank 66
Members: J3-18; primary J3-18
Why together: Single question.
Critic's notes: 

## Finding J3-18
Rules: STR-1, MOD-7, IFC-6, MOD-2, MOD-9; file tasks/spec-0.9.9/parts/p05-declarations.md lines 226-228; 445-446; kind undecided_case; severity low
Quote from Hardened_11:
  or named arguments in field order; a field with a default may be omitted. It is `pub` iff every
  field is `pub`.
* `[IFC-6]` *(new in 0.9.15)* **Bounds of a generic function.** A function that is not `pub` writes all

The finder's question:
When fields are `pub(package)`, may another module of the same package call the memberwise constructor? Does a
`pub(package)` generic function write its bounds (as `pub`) or may it leave them to be worked out (as not
`pub`)?

Program:
```
# net/packet.em
pub struct Packet:
    pub kind: u8
    pub(package) seq: u32

[T]
pub(package) fn largest(xs: Span[T]) -> T:
    best = xs[0]
    for x in xs:
        if x > best:
            best = x
    return best

# net/send.em, same package
from net.packet import Packet, largest
fn first() -> Packet:
    return Packet(kind=1, seq=largest([3, 7]))
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.55)
Reason:
STR-1 (p05:227-228) knows two cases, "It is `pub` iff every field is `pub`". With a `pub(package)` field the
constructor is "not pub", which leaves open whether it is private to the module or visible in the package.
MOD-2 (p05:19-21) has three levels, and MOD-7 (p05:34-37) decides only `pub(read)`, which makes the
constructor private to the module. The finder's second question (IFC-6 for a `pub(package)` generic function)
is settled: MOD-2 defines `pub` as "visible to dependants", and MOD-9 (p05:38-41) states the reason as the
package's edge, which a `pub(package)` function does not cross. So a `pub(package)` function counts as not
`pub` and may leave its bounds out.

options:
[{'letter': 'A', 'summary': 'The constructor gets the narrowest visibility among its fields: here
`pub(package)`.', 'code': '    return Packet(kind=1, seq=7)       # allowed anywhere in the `net` package',
'outcome': 'Compiles. Outside the package, `Packet(kind=1, seq=7)` is E1052, naming `seq`. A `pub(read)` field
keeps the constructor private to its module, as today.'}, {'letter': 'B', 'summary': 'Two cases only: every
field `pub` gives a `pub` constructor; anything else is private to the module.', 'code': '# net/packet.em must
add a builder:\n    pub(package) static fn new(kind: u8, seq: u32) -> Packet:\n        return
Packet(kind=kind, seq=seq)\n\n# net/send.em\n    return Packet.new(1, 7)', 'outcome': '`Packet(kind=1, seq=7)`
in send.em is E1052. It compiles once `packet.em` adds the `static fn` above and send.em calls it.'}]

recommendation:
{'letter': 'A', 'reason': "Whoever may see every field may build the value, which is the same reasoning that
already makes a `pub(read)` field's constructor private. It uses the visibility levels MOD-2 already has.",
'checklist': "2 (accepts the correct program); 5 and 6 (one rule across MOD-2's three levels, consistent with
MOD-7); 12 (no builder function forced only because a level is missing from STR-1)."}

