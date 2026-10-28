#$ test: compile-fail
#$ rules: UNS-12, UNS-1
# `[UNS-12]`, `[UNS-1]` — the block an `E3100` gives to write goes around the whole statement the step
# is in: an `elif` cannot be put in a block alone, so the `if` it belongs to is; a statement over two
# lines is put in whole, and its steps on both lines are one diagnostic.

@safety("p points to two readable bytes")
unsafe fn pick(p: *u8, n: int) -> u8:
    if n > 0:
        return 1
    elif p.read() > 0:    #$ error[E3100]: the read through `p` on lines 9 to 12 is this function's own unsafe step, and nothing says why it is sound; put lines 9 to 12 in an `unsafe:` block with a `# SAFETY:` note saying why the read is safe: above line 9 write `# SAFETY:` and the reason, then `unsafe:`, and indent lines 9 to 12 under it
        return 2
    x = (p.read() +    #$ error[E3100]: the reads through `p` on lines 13 to 14 are this function's own unsafe steps
        p.add(1).read())
    return x

fn main():
    pass
