#$ test: compile-fail
#$ rules: ALC-2, TYP-43
## `[ALC-2]` (ODR-175) — implementing `Allocator`, an `unsafe interface`, is `unsafe`: in a plain
## `extend` or in a type's header it is `E3100`, whose help is `unsafe extend`. And `unsafe extend`
## writes only a marker or an `unsafe interface` (`[TYP-43]`): another interface is `E2040`.

from std.mem import Allocator, Layout, Global

struct Plain:
    pass

extend Plain implements Allocator:    #$ error[E3100]: implementing `std.mem.Allocator` is `unsafe`
    fn allocate(self, layout: Layout) -> *mut u8:
        return Global().allocate(layout)

    fn deallocate(self, p: *mut u8, layout: Layout):
        Global().deallocate(p, layout)

    fn reallocate(self, p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return Global().reallocate(p, old, new_size)

struct Header implements Allocator:    #$ error[E3100]: implementing `std.mem.Allocator` is `unsafe`
    n: int

    fn allocate(self, layout: Layout) -> *mut u8:
        return Global().allocate(layout)

    fn deallocate(self, p: *mut u8, layout: Layout):
        Global().deallocate(p, layout)

    fn reallocate(self, p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return Global().reallocate(p, old, new_size)

interface Named:
    fn name(self) -> str

# SAFETY: it keeps the promises the interface states
unsafe extend Plain implements Named:    #$ error[E2040]: `unsafe extend` writes `Send` or `Sync` by hand, or an `unsafe interface`
    fn name(self) -> str:
        return "plain"

fn main():
    pass
