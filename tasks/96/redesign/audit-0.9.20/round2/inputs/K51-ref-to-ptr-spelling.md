# K51-ref-to-ptr-spelling

Question: How is a pointer made from a reference: one library name, two, or the cast?
Severity: low; impact rank 51
Members: J2-11; primary J2-11
Why together: Single question.
Critic's notes: Must come before K52: J8-12 A builds on `ref_to_ptr`, which B removes (see conflicts). Part XVI (p16 430) and J8-08's example use it too.

## Finding J2-11
Rules: IV.5, TYP-26; file tasks/spec-0.9.9/parts/p04-types.md lines 340-342, 867-868; kind contradiction; severity low
Quote from Hardened_11:
`ref_to_ptr(r: T) -> *T` and `ref_to_ptr(ref mut r: T) -> *mut T`, called
`ref_to_ptr(x)` and `ref_to_ptr(ref mut x)`

The finder's question:
Are there two functions named `ref_to_ptr`, chosen by the word at the call? If so, how does that fit 'There is
no overloading', and may a program write the same pair itself?

Program:
```
[T]
fn addr_of(r: T) -> *T:
    return ref_to_ptr(r)

[T]
fn addr_of(ref mut r: T) -> *mut T:      # E1030 by TYP-26, though std does the same
    return ref_to_ptr(ref mut r)

fn main():
    x = 5
    p = ref_to_ptr(x)
    q = ref_to_ptr(ref mut x)
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.7)
Reason:
IV.5 (p04 340-342) declares two library functions named `ref_to_ptr`, differing only in the parameter's word
and the result type. TYP-26 (p04 867-868) says there is no overloading and two functions of one name in one
scope are E1030, with exceptions only for operator implementations and `extend` blocks of different types;
neither covers this, so the library has an overload a program cannot write. The same idea is already spelled
by the safe cast `r as *T` / `r as *mut T` (p04 342-344). ODR-075 (p26 363) kept the name before the redesign;
no verdict.

options:
[{'letter': 'A', 'summary': 'Two names, one function each.', 'code': 'p = ref_to_ptr(pos)\nq =
ref_to_mut_ptr(ref mut pos)', 'outcome': 'Compiles; `ref_to_ptr(ref mut pos)` is refused, its help giving
`ref_to_mut_ptr(ref mut pos)`.'}, {'letter': 'B', 'summary': 'One spelling, the safe cast that exists;
`ref_to_ptr` goes.', 'code': 'p = ref pos as *Array[float]\nq = ref mut pos as *mut Array[float]', 'outcome':
'Compiles; `ref_to_ptr(pos)` is E1010, its help giving `ref pos as *Array[float]`.'}, {'letter': 'C',
'summary': "Functions of one name may differ by a parameter's word, for programs too.", 'code': '[T]\nfn
addr_of(r: T) -> *T:\n    return ref_to_ptr(r)\n\n[T]\nfn addr_of(ref mut r: T) -> *mut T:\n    return
ref_to_ptr(ref mut r)', 'outcome': "Compiles; the call's word picks the function. Overloading by word becomes
part of the language."}]

recommendation:
{'letter': 'B', 'reason': 'The idea already has a spelling, the safe cast, so the second spelling goes and the
library keeps no overload a program cannot write.', 'checklist': '5 (said once), 11 (the existing spelling
kept, the older one removed), 13 (no library-only path)'}

