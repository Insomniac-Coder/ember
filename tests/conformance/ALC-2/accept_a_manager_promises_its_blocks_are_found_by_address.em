#$ test: run-pass
#$ rules: ALC-2, TYP-43
#$ stdout: 65536 true
## `[ALC-2]` (verdict 107) — `std.mem` declares `pub unsafe interface FoundByAddress`, with no
## methods, and `const PIECE: usize = 65536`; implementing it is `unsafe extend PoolArena implements
## FoundByAddress:`, the promise that every block lies in a piece starting at a multiple of `PIECE`
## whose first word holds the instance's address.

import std.mem
from std.mem import Allocator, FoundByAddress, Layout

struct Pool:
    pass

# SAFETY: it hands out no block at all.
unsafe extend Pool implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        return null[*mut u8]()

    @safety("`p` came from this manager's `allocate` or `reallocate` with `layout`, and nothing uses it after")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        pass

    @safety("`p` came from this manager's `allocate` or `reallocate` with `old`, and nothing uses it after; only the result is used")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return null[*mut u8]()

# SAFETY: it hands out no block, so every block it hands out lies in a piece.
unsafe extend Pool implements FoundByAddress:
    pass

[A: needs FoundByAddress]
fn found(a: A) -> bool:
    return true

fn main():
    println(mem.PIECE, found(Pool()))
