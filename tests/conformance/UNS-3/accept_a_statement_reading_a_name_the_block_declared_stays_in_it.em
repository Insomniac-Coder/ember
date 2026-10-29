#$ test: compile-pass
#$ rules: UNS-3
# D-929 — a statement of an `unsafe:` block that reads a name an earlier statement of the block
# declared stays in the block: the name ends with the block, so moving the statement out would not
# compile, and `L3010` does not name it as needing no `unsafe`.

fn main():
    values: Array[u8] = [7, 8]
    # SAFETY: `values` holds two bytes
    unsafe:
        first = values.as_span().as_ptr().read()
        println(first)
