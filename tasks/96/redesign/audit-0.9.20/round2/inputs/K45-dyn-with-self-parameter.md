# K45-dyn-with-self-parameter

Question: Can an interface whose method takes another Self be used as dyn?
Severity: medium; impact rank 45
Members: J2-08; primary J2-08
Why together: Single question.
Critic's notes: 

## Finding J2-08
Rules: TYP-22, TYP-36; file tasks/spec-0.9.9/parts/p04-types.md lines 785-789, 691-693; kind undecided_case; severity medium
Quote from Hardened_11:
An interface is usable as `dyn` only if every method has an object
  (none is a `static fn`), does not return `Self` by value (methods marked `where Self: needs Sized`
  excepted), and is not generic

The finder's question:
Is an interface whose method takes another `Self` (`fn hits(other: Self) -> bool`) usable as `dyn`, and if so,
what does the call do when the two values are of different kinds? Is `owns dyn I` `Ord` when `I` requires
`Ord`?

Program:
```
interface Collider:
    fn hits(other: Self) -> bool

struct Circle implements Collider:
    r: float
    fn hits(other: Circle) -> bool:
        return this.r + other.r > 1.0

struct Box2 implements Collider:
    w: float
    fn hits(other: Box2) -> bool:
        return this.w + other.w > 1.0

fn check(a: dyn Collider, b: dyn Collider) -> bool:
    return a.hits(b)              # compiles? a Circle given a Box2?

fn main():
    println(check(Circle(r=0.5), Box2(w=2.0)))
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.7)
Reason:
TYP-22 (p04 785-789) makes an interface unusable as `dyn` only for a static method, a method returning `Self`
by value, or a generic method; a method taking a `Self` parameter passes. Nothing says what `a.overlaps(b)`
does when two `dyn` values are of different kinds. TYP-36 (p04 691-693) settles this for Eq only ('values of
two different kinds are never equal', a method-table entry) and says nothing of Ord, so sorting an `Array[owns
dyn Shape]` with `interface Shape: needs Ord:` is undecided too. No verdict.

options:
[{'letter': 'A', 'summary': 'A method with a `Self` parameter makes the interface unusable as `dyn` (E2050),
as returning `Self` does; `where Self: needs Sized` on that method keeps the rest usable. Eq keeps its stated
answer (two kinds are never equal, which is true); an interface requiring Ord is not usable as `dyn`, since no
order between two kinds is true.', 'code': 'interface Collider:\n    fn overlaps(other: Self) -> bool where
Self: needs Sized   # only for known kinds', 'outcome': 'As written, `check`\'s `dyn Collider` is E2050
("`overlaps` takes another `Self`, and two `dyn Collider` values may be of different kinds"). With the marked
line it compiles; `a.overlaps(b)` on `dyn` values is refused, and the program calls it on known kinds or
writes a method taking `other: dyn Collider`.'}, {'letter': 'B', 'summary': 'Allowed; the call checks that
both values are of one kind and stops the program otherwise, one compare of type indices.', 'code': 'fn
check(a: dyn Collider, b: dyn Collider) -> bool:\n    return a.overlaps(b)', 'outcome': 'Compiles; stops:
"`a.overlaps(b)` on line 17: `a` is a `Circle` and `b` an `Aabb`". A Circle with a Circle runs.'}]

recommendation:
{'letter': 'A', 'reason': 'It decides the program when compiling, with the same rule already applied to a
method returning `Self`; Eq stays the one case where an answer across kinds is true.', 'checklist': "3
(compile time first), 6 (the nearest rule, returning Self; Eq's exception exists because its answer is true),
8 (no stop the programmer did not see coming)"}

