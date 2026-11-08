# K50-adopt-ends-by-own-destructor

Question: What does adopt end an object with: taking back into_raw(), and a C++ object from a raw pointer?
Severity: medium; impact rank 50
Members: J8-13, J8-16; primary J8-13
Why together: Both decide what `adopt` uses when no destroys(f) is written: an Ember class's drop (J8-13) or a C++ class's delete (J8-16). One rule ('the way the type's own declaration ends it') answers both.
Critic's notes: 

## Finding J8-13
Rules: FFI-54, FFI-23, FFI-36a, FFI-11; file tasks/spec-0.9.9/parts/p16-ffi.md lines 545-561; kind undecided_case; severity medium
Quote from Hardened_11:
contract says `owns` (the name ends and Ember frees nothing, `[FFI-11]`) or, for a C function whose
  declaration cannot say so, given with `into_raw()` inside `unsafe:` (`[FFI-54]`); C gives a foreign
  object back through `adopt` inside `unsafe:` (`[FFI-36a]`).

The finder's question:
Once an Ember class object or `owns T` of an Ember type has gone to C with `into_raw()` (or an `owns`
contract), how does Ember get it back and end it? `adopt` reads a foreign type's `destroys(f)`, and an Ember
class has none. Can an `ObjectPtr[C]` that C hands back as user data be used as the object at all?

Program:
```
class Game:
    score: i32 = 0

unsafe extern "C":
    @safety("`p` stays valid while the window lives")
    unsafe fn set_user_pointer(p: ObjectPtr[Game])
    @safety("the window has a user pointer")
    unsafe fn get_user_pointer() -> ObjectPtr[Game]

fn main():
    g = Game()
    # SAFETY: the window keeps it until close
    unsafe:
        set_user_pointer(g.into_raw())
        p = get_user_pointer()
        # p.score? adopt[Game](p)? the game is never ended: a leak
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.55)
Reason:
FFI-54 (p16-ffi.md 555-561) and verdict 91 let `into_raw()` give C an `owns T`'s `*mut T` or a class object's
`ObjectPtr[C]`, and say a mistake "leaks it or frees it twice". No operation takes such a value back. `adopt`
(FFI-36a, 386-389) reads the destructor from `@ffi(destroys(f))` on a foreign `type T`, which an Ember class
has none of. `ObjectPtr[C]` (FFI-23, 548-550) is defined only as a `Copy`, comparable address. So an Ember
object given to C as user data (the GLFW `glfwSetWindowUserPointer` pattern) can never be ended. JOB.md and
decisions.md have no `from_raw`.

options:
[{'letter': 'A', 'summary': "`adopt`, in `unsafe:`, also takes back what `into_raw()` gave: an `ObjectPtr[C]`
gives the class object's owner, and the `*mut T` of an `owns T` gives the `owns T`. Ember then ends it as
usual.", 'code': "    # SAFETY: the pointer C hands back is the one `into_raw()` gave\n    unsafe:\n
back = std.ffi.adopt[Game](take_user_pointer())\n    println(back.score)      # `back` ends at the end of
`main`, running Game's drop", 'outcome': 'Compiles. It prints 0 and frees the game at the end of `main`.
Taking the same address back twice is caught as adopting one object twice is (a panic naming both sites).'},
{'letter': 'B', 'summary': "A separate built-in on the type, Rust's spelling.", 'code': '    unsafe:\n
back = Game.from_raw(take_user_pointer())', 'outcome': 'Same as A. The language gets a second word for taking
ownership of an address.'}, {'letter': 'C', 'summary': "There is no way back: what `into_raw()` gives is C's
for good.", 'code': '    unsafe:\n        p = take_user_pointer()', 'outcome': 'Compiles. The game is never
ended (a leak, which the `debug` leak report lists), and any object kept as user data leaks.'}]

recommendation:
{'letter': 'A', 'reason': '`adopt` already means "take ownership of an object C hands back, in unsafe code",
so taking back what `into_raw()` gave reuses that one word, and its existing twice-adopted check applies.',
'checklist': '11 (`adopt` reused, no second word), 2 (the user-data pattern frees its object), 5, 7'}

## Finding J8-16
Rules: FFI-36, FFI-36a, FFI-32; file tasks/spec-0.9.9/parts/p21-annex-c.md lines 118-120; p16-ffi.md 386-393; kind undecided_case; severity low
Quote from Hardened_11:
* `[FFI-36]` *(changed in 0.9.15)* A C++ function returning a raw pointer with no ownership contract
  yields `*mut T`; `ffi.adopt[T](p)` (`unsafe`) turns it into an `owns T` ended through the contract's
  destructor.

The finder's question:
What ends an `owns T` adopted from a raw pointer to an imported C++ class whose overlay states no
`destroys(f)`? Is it the class's destructor thunk plus `delete`, is it refused, or does it need an overlay
entry?

Program:
```
// factory.hpp: Enemy* make_enemy(int kind);   // caller deletes
import cpp "factory.hpp" with (project="game") as f

fn spawn() -> owns f.Enemy:
    # SAFETY: make_enemy returns a new Enemy the caller deletes
    unsafe:
        p = f.make_enemy(3)?
        return std.ffi.adopt[f.Enemy](p)    # ended by `delete`? E5002 for no destroys?
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.5)
Reason:
FFI-36 (p21-annex-c.md 118-120) turns a C++ raw-pointer result into an `owns T` "ended through the contract's
destructor". FFI-36a (p16-ffi.md 386-389) reads that destructor from `@ffi(destroys(f))` on `type T`. A C++
class imported with no overlay has no such word. FFI-32 (74-80) ends a by-value opaque result "by the class's
destructor", and FFI-17a's `std::unique_ptr` row (63) ends one "through the deleter", but nothing says whether
a C++ class's own destructor (with `delete`) counts as "the contract's destructor" for `adopt`, or whether
`adopt` is refused without a `destroys(f)`. JOB.md has no verdict on this.

options:
[{'letter': 'A', 'summary': "For an imported C++ class with no `destroys(f)`, the adopted owner's end runs
C++'s `delete` (its destructor and its memory release) through a generated thunk, as `std::unique_ptr`'s
default deleter does. A class with no public destructor is refused.", 'code': '        return
adopt[f.Enemy](p)', 'outcome': "Compiles. `e`'s end runs `delete` on the enemy. An overlay's
`@ffi(destroys(free_enemy))` replaces `delete` for an object made some other way."}, {'letter': 'B',
'summary': '`adopt` needs a `destroys(f)` stated in the overlay, for C++ as for C.', 'code': 'unsafe overlay
cpp "factory.hpp":\n    @ffi(destroys(destroy_enemy))\n    type Enemy', 'outcome': 'The example without the
overlay is E5002, naming the missing `destroys`. C++ classes need a free function to be written for them.'}]

recommendation:
{'letter': 'A', 'reason': "`delete` is what a C++ class's own declaration says ends it, and Ember already uses
that for by-value results and `unique_ptr`. So `adopt` follows the same rule with nothing extra to write, and
an overlay can still say otherwise.", 'checklist': '5 and 6 (one rule with FFI-32 and FFI-17a), 4, 2'}

