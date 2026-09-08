# Appendix A — Syntax quick reference

Generated from `docs/spec-source/appendix-a.em`, a conformance fixture that compiles (`[TST-6]`). Every
construct below parses under Part III; nothing here is illustrative shorthand.

```ember
#! language "0.5"
import std.io                              # namespace import
from std.math import Vec3, sin as sine     # item import
import c "vulkan/vulkan.h" with (overlay="overlays/vulkan.em")

const MAX: u32 = 1024                      # compile-time constant
static HITS: Atomic[u64] = Atomic(0)       # global (Sync)

@derive(Copy, Debug, Eq)                   # derives
@layout(c)
pub struct Vec3:                           # value type
    pub x: f32
    pub y: f32
    pub z: f32 = 0.0                       # field default
    fn length(self) -> f32: pass           # borrowed receiver
    fn scale(mut self, k: f32): pass       # mutable receiver
    fn into_array(owned self) -> [f32; 3]: pass   # consuming receiver

pub open class Script:                     # reference type, subclassable
    let entity: Entity                     # immutable field
    pub(read) health: f32 = 100.0          # public read, private write
    fn init(mut self, entity: Entity): self.entity = entity
    virtual fn on_update(mut self, dt: f32): pass

class Door(Script):                        # single inheritance
    angle: f32 = 0.0
    override fn on_update(mut self, dt: f32): self.angle += dt

enum Shape:                                # tagged union
    Circle(r: f32)
    Rect(w: f32, h: f32)
    Empty

interface Drawable:
    fn draw(self, mut cmd: CommandList)

extend Vec3 implements Display:            # external impl
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]: pass

fn area(s: Shape) -> f32:                  # params: borrowed by default
    return match s:                        # expression `match`: arms are expressions
        Circle(r) => PI * r * r
        Rect(w, h) => w * h
        Empty => 0.0

fn fill(mut buf: MutSpan[f32], v: f32):    # `mut` = inout
    for x in buf.iter_mut(): x = v         # ref mut local writes through

fn consume(owned xs: Array[i32]) -> usize: return xs.len()   # `owned` = move

fn parse(s: str) -> Result[i32, ParseError]:
    n = s.trim().parse[i32]()?             # ? propagates
    return Ok(n)

@noalloc @simd
fn integrate(mut p: SoA[Particle], dt: f32):        # contract + hint
    for i in 0..p.len(): p.velocity[i] += GRAVITY * dt

## Statements live in a function: `file` admits only items (`[GRM-2]`).
fn demo(n: usize, out: MutSpan[f32], inp: Span[f32], m: Mutex[Array[i32]]):
    @parallel(chunk=256)                   # statement attribute (`[ATT-2]`, `[ATT-3]`)
    for i in 0..n: out[i] = f(inp[i])      # parallel loop

    frame = Arena.with_capacity(16 * MB)   # arena
    cmds  = frame.alloc_array[Cmd](count)  # MutSpan tied to frame

    double = fn(x: f32) => x * 2.0         # closure (borrowing)
    task   = owned fn() => run(job)        # escaping closure

    with lock = m.lock(): lock.push(1)     # scoped guard
    defer: cleanup()                       # runs at scope exit
    unsafe: ptr.write(0)                   # unsafe block
    comptime: assert(size_of[Vec3]() == 12)  # compile-time check
    x = a if cond else b                   # ternary
    h: Option[Player] = None
    if Some(p) = h: p.health -= 1          # pattern condition
```

*End of specification.*
