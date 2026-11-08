# K69-soa-flattened-field

Question: What is ps.pos when pos is split into x, y, z columns?
Severity: low; impact rank 69
Members: J7-13; primary J7-13
Why together: Single question.
Critic's notes: 

## Finding J7-13
Rules: SOA-1, SOA-2, SOA-6, SIMD-3; file tasks/spec-0.9.9/parts/p12-dod.md lines 35-38; kind undecided_case; severity low
Quote from Hardened_11:
A field marked `@soa(flatten)` whose type is a struct is / split into one column per field of its own
(`ps.pos.x`); otherwise a struct field is one column. / * `[SOA-2]` *(changed in 0.9.15)* `ps.f` names the
column of field `f`: a view of that column's elements, a `MutSpan[F]`,

The finder's question:
For a field marked `@soa(flatten)`, what is `ps.pos`: a `MutSpan[Vec3]` (impossible, as the Vec3s are not
contiguous), an error, or a group of columns? And what are `ps[i].pos` and `ps[i].pos = v`?

Program:
```
from std.math import Vec3

struct Particle:
    @soa(flatten)
    pos: Vec3
    vel: Vec3

fn integrate(ref mut pos: MutSpan[Vec3], vel: Span[Vec3], dt: f32):
    for i in 0..pos.len():
        pos[i] += vel[i] * dt

fn main():
    ps = SoA[Particle].with_capacity(1024)
    integrate(ref mut ps.pos, ps.vel, 0.016)   # pos is flattened: does this compile?
    ps[0].pos = Vec3.ZERO                       # one place or three?
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.6)
Reason:
SOA-1 (p12 35-36) splits a `@soa(flatten)` field into one column per field of its own (`ps.pos.x`). SOA-2 (p12
37) defines `ps.f` as 'the column of field `f` ... a `MutSpan[F]`', and SOA-6 (p12 51-52) defines `ps[i].f` as
'a place in column `f`'. A flattened field has no column of its own, so neither rule says what `ps.pos`,
`ps[i].pos` or `ps[i].pos = v` are. Searched SOA-1 to SOA-7, SIMD-3 (c), the attribute table, and HR-13.

options:
[{'letter': 'A', 'summary': '`ps.pos` is no column; `ps[i].pos` gathers and scatters its columns', 'code':
'integrate(ref mut ps.pos, ps.vel, 0.016)\n# refused: "`pos` is split into the columns `ps.pos.x`, `ps.pos.y`
and `ps.pos.z` (`@soa(flatten)`,\n#  line 4), so there is no column of `Vec3`; pass the columns: `ps.pos.x`,
..."\nps[0].pos = Vec3.ZERO        # accepted: writes ps.pos.x[0], ps.pos.y[0], ps.pos.z[0]', 'outcome': "The
`integrate` line does not compile. `ps[0].pos` reads the three columns into a `Vec3`, and `ps[0].pos = v`
writes v's three fields into them, as `ps[i] = v` scatters a whole value."}, {'letter': 'B', 'summary':
'`ps.pos` is a view of its columns together, a new view type', 'code': 'fn integrate(ref mut pos:
SoASpan[Vec3], vel: Span[Vec3], dt: f32):   # a new type: Vec3s stored as columns\n    for i in
0..pos.len():\n        pos[i] += vel[i] * dt\n\nintegrate(ref mut ps.pos, ps.vel, 0.016)', 'outcome':
'Compiles with the new view type. A function taking `MutSpan[Vec3]` still cannot take `ps.pos`.'}]

recommendation:
{'letter': 'A', 'reason': "It adds no new type: the field's own columns already exist, and an element's field
behaves as an element does (gather on read, scatter on write). B adds a view kind for one attribute.",
'checklist': '5 (`ps[i].pos` behaves as `ps[i]` does), 7 (no special type for one attribute), 9 (the message
names the columns to use), 13 (no extra path)'}

