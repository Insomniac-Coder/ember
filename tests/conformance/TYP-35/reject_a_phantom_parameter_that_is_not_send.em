#$ test: compile-fail
#$ rules: TYP-35
## `[TYP-35]` - a phantom parameter counts for `Send` as a field of that type would: a marker that is not `Send`
## makes the type thread-confined without a field.

[Tag]
struct Handle:
    index: u32

[T: needs Send]
fn send_it(x: T):
    pass

fn main():
    ok = Handle[int](1)
    send_it(ok)
    confined = Handle[*mut u8](2)
    send_it(confined)                #$ error[E2040]: `Handle[*mut u8]` does not implement `Send`, which `T` requires
