#$ test: run-pass
#$ rules: CORO-9, UNS-8
#$ profiles: debug, release
#$ stdout: 1
#$ 7
# `[CORO-9]` — `unsafe` code may build a frame that points into itself, and states why it is
# sound in its safety note: here the frame stays where the loop put it while the pointer is used.

# SAFETY: the pointer into the frame is read only while the frame has not moved since `next` made it.
unsafe gen fn pinned() -> Generator[int]:
    n = 5
    p = ref_to_ptr(ref n)
    yield 1
    n = 7
    unsafe:  # SAFETY: `p` points at `n`, in the frame, which has not moved since `p` was made
        yield read(p, 0)

fn main():
    unsafe:  # SAFETY: the loop keeps the frame in one place from its first `next` to its last
        for x in pinned():
            println(x)
