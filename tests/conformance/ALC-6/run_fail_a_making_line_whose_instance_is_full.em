#$ test: run-fail
#$ rules: ALC-6
#$ panics: out of memory: `owns Cmd(id=1) using fixed` (line 31) asked `fixed` for 8 bytes
## `[ALC-6]` — when the instance's `allocate` returns null, the compiler's own call panics, "out of
## memory: `owns Cmd(id=1) using fixed` (line 8) asked `fixed` for 16 bytes", in the program's words.

from std.mem import Allocator, Layout

struct Full:
    asked: int = 0

# SAFETY: it hands out no block at all.
unsafe extend Full implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.asked += 1
        return null[*mut u8]()

    @safety("`p` came from this manager's `allocate`")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        pass

    @safety("`p` came from this manager's `allocate`")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return null[*mut u8]()

struct Cmd:
    id: int

fn main():
    fixed = Full()
    c = owns Cmd(id=1) using fixed
    println(c.id)
