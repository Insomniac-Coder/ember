#$ test: compile-fail
#$ rules: UNS-1
# `[UNS-1]` — the block an `E3100` gives to write goes around the whole statement the step is in: an
# `elif` cannot be put in a block alone, so the `if` it belongs to is; a statement over two lines is
# put in whole, and its steps on both lines are one diagnostic.
#$ help: put lines 9 to 12 in an `unsafe:` block

fn pick(p: *u8, n: int) -> u8:
    if n > 0:
        return 1
    elif p.read() > 0:    #$ error[E3100]: `read` through a raw pointer needs an `unsafe` block
        return 2
    x = (p.read() +    #$ error[E3100]: the reads through `p` on lines 13 to 14 are unsafe steps outside unsafe code
        p.add(1).read())
    return x

fn main():
    pass
