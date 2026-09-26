#$ test: run-pass
#$ rules: ARN-11, ARN-3, DRV-1
#$ stdout: 8 7 0.0 0
# `[ARN-11]` (ODR-064) — a struct is `Zeroable` when it declares
# `@derive(Zeroable)` and every field is, which the compiler proves; a
# generic one is, for each instance whose fields are. This is §IX.2's
# example, which before SP-017 called `alloc_array` on a struct with no
# `Default` (`E2040`).

@derive(Zeroable)
struct Cmd:
    id: int
    cost: f32

@derive(Zeroable)
struct Wrap[T]:
    item: T

fn build(frame: Arena, n: int) -> MutSpan[Cmd]:
    cmds = frame.alloc_zeroed[Cmd](n)
    for i in 0..n:
        cmds[i].id = i
    return cmds

fn main():
    frame = Arena.with_capacity(64 * 1024)
    cmds = build(frame, 8)
    wraps = frame.alloc_zeroed[Wrap[u16]](2)
    println(cmds.len(), cmds[7].id, cmds[3].cost, wraps[1].item)
