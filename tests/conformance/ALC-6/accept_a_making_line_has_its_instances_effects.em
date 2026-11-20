#$ test: run-pass
#$ rules: ALC-6
#$ stdout: 1 2
## `[ALC-6]` — a line that makes a value `using x`, and its end, carry the effects of the `Allocator`
## methods it calls on `x`'s type: a manager whose `allocate` and `deallocate` allocate nothing makes
## values in `@noalloc` code.

from std.mem import Allocator, Layout

struct Bump:
    slots: [int; 8] = [0; 8]
    used: int = 0

# SAFETY: each block is a slot of the array, aligned for an `int`, given out once.
unsafe extend Bump implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        if this.used == 8 or layout.size > 8 or layout.align > 8:
            return null[*mut u8]()
        this.used += 1
        # SAFETY: a slot's bytes are the block, aligned for an `int`.
        unsafe:
            return ref_to_ptr(ref mut this.slots[this.used - 1]) as *mut u8

    @safety("`p` came from this manager's `allocate`")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        pass

    @safety("`p` came from this manager's `allocate`")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return null[*mut u8]()

struct Cmd:
    id: int

@noalloc
fn make(ref mut bump: Bump, id: int) -> owns Cmd using Bump:
    return owns Cmd(id=id) using bump

fn main():
    bump = Bump()
    a = make(ref mut bump, 1)
    b = make(ref mut bump, 2)
    println(a.id, b.id)
