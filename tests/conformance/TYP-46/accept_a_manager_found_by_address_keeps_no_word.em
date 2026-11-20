#$ test: run-pass
#$ rules: ALC-2, ALC-1, TYP-46
#$ profiles: debug, development, release
#$ stdout: 7 8 true
#$ stdout: 2
## `[ALC-2]` (verdict 107) — a manager that implements `FoundByAddress` promises that every block it
## hands out lies in a piece starting at a multiple of `PIECE` bytes whose first word holds the
## instance's address; a value made in it keeps no word, and its end reads the instance from the start
## of its piece, one mask and one load: both ends here reach the pool's `deallocate`.

from std.mem import Allocator, FoundByAddress, Layout, Global, PIECE

struct PoolArena:
    piece: *mut u8
    used: usize
    given_back: int

# SAFETY: every block lies in the one piece, which starts at a multiple of `PIECE` and holds the pool's address first.
unsafe extend PoolArena implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        aligned = (this.used + layout.align - 1) & ~(layout.align - 1)
        if aligned + layout.size > PIECE:
            return null[*mut u8]()
        this.used = aligned + layout.size
        # SAFETY: `aligned` is inside the piece.
        unsafe:
            return this.piece.add(aligned)

    @safety("`p` came from this pool's `allocate`")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        this.given_back += 1

    @safety("`p` came from this pool's `allocate`")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return null[*mut u8]()

# SAFETY: its one piece holds its address in its first word.
unsafe extend PoolArena implements FoundByAddress:
    pass

struct Cmd:
    id: int

struct Holder:
    cmd: owns Cmd using PoolArena

fn main():
    piece = Global().allocate(Layout(PIECE, PIECE))
    pool = PoolArena(piece=piece, used=64, given_back=0)
    # SAFETY: the piece is `PIECE` bytes, aligned to `PIECE`; its first word is the pool's address.
    unsafe:
        (piece as *mut *mut PoolArena).write(ref_to_ptr(ref mut pool))
    a = owns Cmd(id=7) using pool
    h = Holder(cmd=owns Cmd(id=8) using pool)
    println(a.id, h.cmd.id, size_of[Holder]() == size_of[usize]())
    mem.drop(a)
    mem.drop(h)
    println(pool.given_back)
    # SAFETY: every value made in the pool has ended.
    unsafe:
        Global().deallocate(pool.piece, Layout(PIECE, PIECE))
