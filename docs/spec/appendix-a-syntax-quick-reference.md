# Appendix A — Syntax quick reference

```ember
#! language "0.2"
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
    fn length(self) -> f32: ...            # borrowed receiver
    fn scale(mut self, k: f32): ...        # mutable receiver
    fn into_array(owned self) -> [f32; 3]: ...   # consuming receiver

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
    fn fmt(self, mut f: Formatter) -> Result[void, FmtError]: ...

fn area(s: Shape) -> f32:                  # params: borrowed by default
    match s:
        Circle(r) => return PI * r * r
        Rect(w, h) => return w * h
        Empty => return 0

fn fill(mut buf: MutSpan[f32], v: f32):    # `mut` = inout
    for x in buf.iter_mut(): x = v         # ref mut local writes through

fn consume(owned xs: Array[i32]) -> usize: return xs.len()   # `owned` = move

fn parse(s: str) -> Result[i32, ParseError]:
    n = s.trim().parse[i32]()?             # ? propagates
    return Ok(n)

@noalloc @simd
fn integrate(mut p: SoA[Particle], dt: f32):        # contract + hint
    for i in 0..p.len(): p.velocity[i] += GRAVITY * dt

@parallel(chunk=256)
for i in 0..n: out[i] = f(inp[i])          # parallel loop

frame = Arena.with_capacity(16 * MB)       # arena
cmds  = frame.alloc_array[Cmd](count)      # MutSpan tied to frame

double = fn(x: f32) => x * 2.0             # closure (borrowing)
task   = owned fn() => run(job)            # escaping closure

with lock = m.lock(): lock.push(1)         # scoped guard
defer: cleanup()                           # runs at scope exit
unsafe: ptr.write(0)                       # unsafe block
comptime: assert size_of[Vec3]() == 12     # compile-time check
x = a if cond else b                       # ternary
h: Option[Player] = None
if Some(p) = h: p.health -= 1              # pattern condition
```

*End of specification.*
