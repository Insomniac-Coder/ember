#$ test: compile-fail
#$ rules: UNS-12, FN-6
# `[UNS-12]` — `drop` runs wherever a value ends, in safe code, so it is never an `unsafe fn`.
# (An `override` of a method that is not `unsafe`, and the implementation of an interface method
# declared without `unsafe`, are refused too; those come with milestone M11 and are not tested here.)

struct Window:
    handle: int

extend Window:
    @safety("the handle is destroyed once")
    unsafe fn drop():    #$ error[E2020]: `drop` runs wherever a `Window` ends, in safe code, so it cannot be an `unsafe fn`; write `fn drop():` and put what needs `unsafe` in it in an `unsafe:` block with a `# SAFETY:` note
        pass

fn main():
    pass
