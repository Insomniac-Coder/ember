#$ test: compile-fail
#$ rules: UNS-12, UNS-1
# `[UNS-12]` (verdict 118) — in an `unsafe fn` a `safe:` block is checked as safe code: an unsafe
# step in it is `E3100`, which says so and gives both ways out, and a statement's steps are one
# `E3100`. An `unsafe:` block inside the `safe:` block is unsafe again.

@safety("`p` points to four writable bytes")
unsafe fn clear(p: *mut u8):
    p.write(0)
    safe:
        p.write(1)    #$ error[E3100]: `p.write(1)` (line 11) is in a `safe:` block, which is checked as safe code; move it out of the `safe:` block, or put line 11 in an `unsafe:` block
        println(p.add(1).read() + p.add(2).read())    #$ error[E3100]: the reads through `p` on line 12 are in a `safe:` block, which is checked as safe code; move them out of the `safe:` block, or put line 12 in an `unsafe:` block
        # SAFETY: the caller promised four writable bytes at `p`
        unsafe:
            p.add(3).write(2)

fn main():
    pass
