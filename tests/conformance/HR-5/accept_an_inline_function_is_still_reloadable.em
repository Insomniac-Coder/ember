#$ test: run-pass
#$ rules: HR-9a, CG-C-3a
#$ profiles: debug, development
#$ stdout: 12
#$ assert-c: contains("int64_t em_add_one__rl(int64_t x) {")
#$ assert-c: contains("{ "em_add_one", (void (*)(void))em_add_one__rl, (void (*)(void))em_add_one,")
#$ assert-c: !contains("EMBER_INLINED EMBER_MAYBE_UNUSED int64_t em_add_one")
# In a reloadable build (this directory's manifest says `reload = "all"`) a function written
# `@inline` is reloadable like any other: it is reached through its thunk and listed in the reload
# manifest, and its `@inline` has no effect there, since an inlined body could not be swapped. A
# build without hot reload inlines it.

@inline
fn add_one(x: int) -> int:
    return x + 1

fn main():
    println(add_one(11))
