# K74-anyerror-core-layer

Question: Can firmware with no heap write Result[int], whose error is AnyError?
Severity: low; impact rank 74
Members: J6-11; primary J6-11
Why together: Single question.
Critic's notes: It touches AnyError like K20, and the two do not conflict.

## Finding J6-11
Rules: STD-6, BLD-11, ERR-9, ERR-8, ERR-11; file tasks/spec-0.9.9/parts/p15-stdlib.md lines 11-15, 54; kind contradiction; severity medium
Quote from Hardened_11:
`std` is layered, and each layer depends only on the ones before it: **core** (scalars,
  `Option`, `Result`, views, fixed-capacity containers, math, SIMD), **alloc** (`Array`, `String`,
  `Map`, `Set`, and the heap blocks of `owns` and `shares` values)

The finder's question:
Can a package with `layers = ["core"]` write `-> Result[int]` (whose error type defaults to `AnyError`), or
`AnyError` at all?

Program:
```
# ember.toml: [package] layers = ["core"]

fn read_sensor(raw: Span[u8]) -> Result[int]:      # E defaults to AnyError
    if raw.len() < 2:
        return Err(AnyError.msg(String("short frame")))
    return Ok(raw[0] as int)
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.6)
Reason:
The module table (p15 54) puts AnyError in std.core and STD-6 (p15 11-15) lets a `layers = ["core"]` package
use only core, while AnyError holds an `owns dyn Error` (ERR-8, p13 93-94), a heap block that STD-6 places in
the alloc layer, and converting into it allocates (ERR-11, p13 106-107). ERR-9 (p13 48-49) makes AnyError the
default error of every `Result[T]`. BLD-11 (spec 7931) bars a core-only package from alloc. So a core-only
package's `-> Result[int]` is either refused (by no stated rule) or core allocates. ODR-151 covers profiles,
not this. No verdict.

options:
[{'letter': 'A', 'summary': 'AnyError belongs to the layer with a heap', 'code': 'pub fn read_sensor(raw:
Span[u8]) -> Result[int]:', 'outcome': 'A compile error in a core-only package: "`Result[int]` fails with
`AnyError`, which keeps its error on the heap, and this package uses only `core`; name the error type: `->
Result[int, FrameError]`". With the error type named, it compiles and allocates nothing.'}, {'letter': 'B',
'summary': 'AnyError stays in core, and its heap block is allowed there', 'code': 'pub fn read_sensor(raw:
Span[u8]) -> Result[int]:', 'outcome': 'Compiles; the core layer then allocates on every failure that reaches
`?` into AnyError.'}]

recommendation:
{'letter': 'A', 'reason': 'A core-only package is the no-heap promise for firmware and kernels; A keeps one
rule for every heap block and the message gives a line that compiles.', 'checklist': "2, 5, 6 (STD-6's layer
of heap blocks), 9, 13"}

