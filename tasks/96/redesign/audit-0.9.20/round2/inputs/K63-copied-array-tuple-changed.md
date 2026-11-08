# K63-copied-array-tuple-changed

Question: Is changing a copied fixed array or tuple refused, as for a copied struct?
Severity: low; impact rank 63
Members: J4-11; primary J4-11
Why together: Single question.
Critic's notes: 

## Finding J4-11
Rules: OWN-7; file tasks/spec-0.9.9/parts/p07-ownership.md lines 164-166; kind inconsistency; severity low
Quote from Hardened_11:
A `Copy` struct copied with `=` from a name that is still used afterwards is not then
  changed through the new name (`E3023`): the line says `copy of` (a copy of your own) or `ref mut`
  (the original), so that no copy is changed by mistake.

The finder's question:
Is `q[0] += 1.0` on a copied `[float; 3]`, or `u.0 += 1` on a copied `(int, int)`, E3023 as `b.x += 1.0` is on
a copied `Copy` struct?

Program:
```
@derive(Copy)
struct Vec3:
    x: float
    y: float
    z: float

fn main():
    a = Vec3(x=0.0, y=0.0, z=0.0)
    b = a
    b.x += 1.0          # E3023 ([OWN-7]): `a` is used below
    p: [float; 3] = [0.0, 0.0, 0.0]
    q = p
    q[0] += 1.0         # a Copy fixed array: E3023 too, or allowed?
    t = (0, 0)
    u = t
    u.0 += 1            # a Copy tuple: E3023 too, or allowed?
    println(a.x, p[0], t.0)
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.5)
Reason:
[OWN-7]'s rule names only "a `Copy` struct". Tuples and fixed arrays of `Copy` elements are `Copy` too (p04
263-264), and their parts can be changed through a copy in the same way. The rule comes from the redesign's R1
(run 4 v5 rules.md 111-114), not from an owner verdict. Nothing says whether its reason, "so that no copy is
changed by mistake", reaches them.

options:
[{'letter': 'A', 'summary': 'One rule for every `Copy` value whose parts can be changed: a struct, a tuple, a
fixed array (not numbers, `bool`, `char` or views).', 'code': 'pos = copy of origin      # a fixed array of
your own, changed freely\npos[0] += 1.0\ns2 = copy of span\ns2.0 += 1', 'outcome': '`pos = origin` then
`pos[0] += 1.0` is refused like the struct (E3023, the help giving `pos = copy of origin` or `pos = ref mut
origin`). Written as above, the program compiles and prints 0.0 0.0 0.'}, {'letter': 'B', 'summary': 'Only
structs, as the rule is written.', 'code': 'pos = origin\npos[0] += 1.0\ns2 = span\ns2.0 += 1', 'outcome':
'Compiles and prints 0.0 0.0 0; a copied fixed array or tuple may be changed through the new name with no
word.'}]

recommendation:
{'letter': 'A', 'reason': "The rule's reason holds for a fixed array most of all: in Python, `=` makes a list
shared, so a programmer may well think `pos` and `origin` are one array. One rule wherever the same thing can
happen.", 'checklist': "5 (one rule for every type with parts), 6 (nearest rule: the struct's), 7"}

