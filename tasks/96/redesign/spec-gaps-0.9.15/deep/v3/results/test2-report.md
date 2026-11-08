# Report: test2 (made by deep/v3/make_report.py; nothing here is in the specification)

| cluster | status | agents | decisions | holes left | tests |
|---|---|---:|---:|---:|---:|
| K07-raw-pointer-struct-safe-call | unresolved | 7 | 0 | 5 | 81 |
| K20-anyerror-send | unresolved | 7 | 1 | 3 | 67 |
| K21-cpp-overloads | unresolved | 7 | 4 | 9 | 90 |

## K07-raw-pointer-struct-safe-call (P04; J8-08): **unresolved**, 7 agents

Outcome: `decided`. Intent: Have a C routine clear a pixel buffer that the Ember program owns, where the buffer is described to C by a pointer and a length in a struct.
The rule that decides it: FFI-10 (p16-ffi.md 82-84): "A raw `*T` can be declared for an unsafe call, but complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is `E5002`. (Recorded as ODR-073, p26-appx-h.md 361, whose ruling reads: "complete `@ffi` facts alone never make that raw pointer safe to pass from Safe code", docs/OWNER-QUEUE.md 3682-3684, chosen because "a safe caller may pass a null, dangling or forged pointer", 3672-3673.)"

### Edit: FFI-10 (change, p16-ffi.md 83)

**Before:**

> complete facts do not make a raw-pointer parameter or result safe to expose; `safe fn` with one is

**After:**

> complete facts do not make a raw-pointer parameter or result safe to expose, nor any raw pointer C reads that safe code may have set (§IV.5): at any depth of an argument or the object, of what an Ember function given to C returns, or of a result safe code may write (`ref mut`, `MutSpan`, `owns`). One held in what C only hands to safe code is allowed: reading through it is unsafe. Every declaration asserting foreign code safe to call keeps to this or, naming where the pointer is held and the line to write, is

Why general: This is FFI-10's own property, in ODR-073's ruling words: complete facts never make a raw pointer "safe to pass from Safe code", because safe code can set one to any address (`ref_to_ptr`, `as_ptr()`, `null[P]()` are safe, §IV.5, SPN-8). The edit judges the property by which way the value goes, not by where it sits in a type. So one set of words covers every type (struct, union, enum payload, fixed array, span element, the value a contracted pointer views), every route into C (an argument, a method's object, a callback's result, C memory that safe code may write through a `ref mut` or `MutSpan` result, the field an `owns` value's destructor reads), and every declaration that asserts foreign code safe (a hand `safe fn`, a C or C++ overlay entry, an `@ffi(safe)` typedef, an `@ffi(std_function)` type), without naming any of them. What C only hands to safe code (a copied result's fields, an `out` value, a callback's parameters) is left alone, because reading through a raw pointer already needs `unsafe` (§IV.5). The rule for a raw-pointer parameter or result itself is kept word for word.

### Edit: FFI-56 (change, p16-ffi.md 598)

**Before:**

> scalar or a `@layout(c)` value type (`E5002` otherwise)

**After:**

> scalar or a `@layout(c)` value type that `[FFI-10]` allows (`E5002` otherwise)

Why general: FFI-56's list is FFI-10's list applied to C function pointers. Pointing it at FFI-10's rule keeps the two lists from contradicting each other. A call through an asserted typedef can then not hand C a raw pointer that safe code set in a parameter's struct, and a struct the pointer returns is judged as any result C hands to safe code.

Readability: Unchanged, word for word. So is a program that only receives C's pointers: SDL's display mode and a qsort-style comparator. The first answer refused the SDL call; the repair does not. Only a binding that hands C a raw pointer which safe code may set changes. It declares `unsafe fn` with `@safety` and writes one wrapper, the existing pattern for a raw-pointer parameter. The caller's line gets shorter: `clear(ref mut frame.as_mut_span())` in place of building a `PixelBuf` by hand.
Cost: None: it is decided when compiling, and nothing runs. The wrapper a binding writes makes the same single C call (FFI-9) and fills the struct from the span's pointer and length, as C code would. To measure: the wrapper call against a direct C call (expected to inline to the same code).

Final check of the last breakers' holes: 5 confirmed real, 0 refuted, 0 already in today's specification (only the confirmed count)

### Holes left (5)

* **undecided**, medium: The edit's own line to write for the asset loader goes wrong either way. On one reading it does not compile (checklist 9), and with it every owned foreign struct that holds a pointer is refused, such as an SDL game's `@ffi(destroys(SDL_FreeSurface)) type SDL_Surface`, whose `pixels` and `format` are raw. On the other, one safe field write brings back the double free (C++: double delete) for which the `returns_owned` form was refused, so that refusal protects nothing and only costs a wrapper. The repair's note that this route is 'outside FFI-10, which judges declarations' is not true: `destroys(f)` is a declaration. Two compilers written from these words disagree on an ordinary SDL binding.

```
// blob.h
#include <stddef.h>
#include <stdint.h>
struct blob { uint8_t* data; size_t len; };
struct blob* blob_new(size_t len);    /* malloc'd blob whose data holds len malloc'd bytes, or NULL */
void blob_free(struct blob* b);       /* free(b->data); free(b); */

# overlays/blob.em
# SAFETY: the entries below match blob.h and its documentation
unsafe overlay c "blob.h":
    @ffi(destroys(blob_free))
    type blob
    @safety("`b` is a blob `blob_new` made, not freed yet")
    unsafe fn blob_free(b)
    @safety("the caller ends the result once, with `blob_free`")
    @ffi(no_callbacks)
    unsafe fn blob_new(len)

# main.em: an asset loader keeps each loaded asset as an owned blob
from std.ffi import adopt
import c "blob.h" with (overlay = "overlays/blob.em") as bl

fn load(len: usize) -> owns bl.blob:
    # SAFETY: `blob_new` gives a fresh blob, checked non-null, that only this owner ends
    unsafe:
        p = bl.blob_new(len)
        assert(p != null[*mut bl.blob]())
        return adopt[bl.blob](p)

fn main():
    asset = load(64)
    println(asset.len)
```

* **cross_rule_clash**, medium: A correct program is refused with no Ember line that compiles: it writes 4 samples into a 4-sample buffer, vouched for in `unsafe:`. The only way out is a C++ helper. Every `std::function` that C++ hands out with a raw pointer in its parameters or result (a filter, a renderer, an allocator hook) becomes uncallable from Ember. The edit closes S1-3's overflow by refusing, but the syntax its refusal needs, an unsafe form of a `std::function` value, is missing (checklist 9 and 12, the lesson of ODR 13).

```
// dsp.hpp
#include <cstddef>
#include <functional>
using RenderFn = std::function<void(float* out, std::size_t frames)>;
RenderFn make_sine(float hz) noexcept;    // the result writes `frames` samples at `out`

# overlays/dsp.em
# SAFETY: the entries below match dsp.hpp and its documentation
unsafe overlay cpp "dsp.hpp":
    @ffi(std_function)
    type RenderFn
    @ffi(no_callbacks)
    fn make_sine(hz)

# main.em: a synthesizer renders one block of a sine voice
import cpp "dsp.hpp" with (overlay = "overlays/dsp.em") as dsp

fn main():
    buf: Array[f32] = [0.0, 0.0, 0.0, 0.0]
    render = dsp.make_sine(440.0)
    # SAFETY: `buf` holds 4 writable samples, and `render` writes `frames` of them
    unsafe:
        render(buf.as_mut_span().as_mut_ptr(), 4)
    println(buf[0])
```

* **needless_refusal**, medium: An ordinary game loop is refused for a line it does not contain: `input.ini_path = ref_to_ptr(ref mut b)`. The danger lives at the write of a pointer field, not at the declaration that gives the view. So every program that takes a writable view of a C struct with any pointer field pays: ImGui's IO block every frame, a window surface, a stream state. The rare dangerous line is what should pay. The edit's readability verdict ("Only a binding that hands C a raw pointer which safe code may set changes") misses these programs. Refusing the view, rather than the pointer-field write, is a choice, and it is made with `decisions: []` under outcome `decided`.

```
// engine.h (a game engine's C API, abridged; cimgui's igGetIO()/igNewFrame() have this shape)
typedef struct frame_input {
    float delta_time;
    float display_w;
    float display_h;
    const char *ini_path;    /* read as NUL-terminated text when the engine saves its settings */
    void *user_data;         /* the host's own pointer, kept for it */
} frame_input;
frame_input *engine_input(void);    /* the engine's input block, alive for the whole program */
void engine_begin_frame(void);      /* reads the input block; may save settings to ini_path */

# overlays/engine.em
unsafe overlay c "engine.h":
    @ffi(no_callbacks)
    fn engine_input() -> borrowed one exclusive from(static)
    @ffi(no_callbacks)
    fn engine_begin_frame()

# main.em: the game loop hands the engine each frame's timing and window size
import c "engine.h" with (overlay = "overlays/engine.em") as en

fn main():
    for _ in 0..3:
        input: ref mut en.frame_input = en.engine_input()
        input.delta_time = 0.016
        input.display_w = 1280.0
        input.display_h = 720.0
        en.engine_begin_frame()
    println("ran 3 frames")
```

* **not_general**, medium: A correct loader that only reads `width` and `height` is refused. The rewrite the message gives then restores exactly the danger the refusal names: an invalid free, `free` on a pointer the program never allocated. Every `returns_owned` binding of a C struct with a pointer field (an image, a JSON node, a font face) gains a wrapper and no safety. The pointer-field write is the one place the edit's property ('safe code may have set') can actually be enforced for an owned foreign value.

```
// image.h (a small image library a game links, abridged)
#include <stdint.h>
typedef struct image { int width; int height; int channels; uint8_t *pixels; } image;
image *image_load(const char *path);   /* NULL when the file cannot be read or decoded */
void image_free(image *img);           /* free(img->pixels); free(img); */

# overlays/image.em
unsafe overlay c "image.h":
    @ffi(destroys(image_free))
    type image
    @safety("`img` is an image `image_load` made, not freed yet")
    unsafe fn image_free(img)
    @ffi(no_callbacks)
    fn image_load(path: borrowed nul_terminated) -> returns_owned(destructor=image_free) one nullable

# main.em: the game reads its splash screen's size
import c "image.h" with (overlay = "overlays/image.em") as im

fn main():
    match im.image_load(c"assets/splash.png"):
        Some(splash) => println(splash.width, splash.height, splash.channels)
        None => println("no splash")
```

* **regression**, medium: An existing conformance test, and FFI-11c's day-one path ("A header can be adopted on the first day with everything behind `unsafe:`, and made safe one function at a time"), change outcome or become undecided. This happens outside the question and with no recorded decision. Two compilers built from these words disagree on an ordinary overlay. Refusing such entries in favour of `unsafe fn` may be what verdicts 130 and 136 point to, but that is a choice reaching FFI-2 and FFI-11c, not one this edit can make silently.

```
// macros.h (tests/conformance/FFI-6b, abridged: HIDDEN_ID, LOG_ALL and twice left out)
typedef struct { int x; int y; } Point;
struct Pair { long long first; long long second; };
#define SQUARE(x) ((x) * (x))
#define SCALE(value, by) ((value) * (by))
#define POINT_SUM(p) ((p).x + (p).y)
#define PAIR_FIRST(p) ((p)->first)
#define IS_EVEN(n) (((n) % 2) == 0)

# overlays/macros.em
unsafe overlay c "macros.h":  # SAFETY(ffi): each entry matches its declaration in the header
    @ffi(macro_fn)
    fn SQUARE(x: i32) -> i32
    @ffi(macro_fn)
    fn SCALE(value: f64, by: f64) -> f64
    @ffi(macro_fn)
    fn POINT_SUM(p: Point) -> i32
    @ffi(macro_fn)
    fn PAIR_FIRST(p: *Pair) -> i64
    @ffi(macro_fn)
    fn IS_EVEN(n: u32) -> bool

# accept_a_declared_macro_is_called_through_its_wrapper.em (run-pass today)
import c "macros.h" with (overlay = "overlays/macros.em")

fn main():
    point = c.Point(x = 3, y = 4)
    pair = c.Pair(first = 7, second = 8)
    first: i64 = 0
    unsafe:  # SAFETY(ffi): `pair` is alive for the call, which only reads it
        first = c.PAIR_FIRST((ref pair) as *c.Pair)
    println(c.SQUARE(5), c.SCALE(1.5, 3.0), c.POINT_SUM(point), first, c.IS_EVEN(6), c.IS_EVEN(7))
```

New gaps found on the way (not this edit's): 11

To verify on the compiler:
* Today's compiler accepts J8-08's program (`safe fn clear_pixels(buf: PixelBuf)` with `data: *mut u8`) with no error.
* After the edit: J8-08's `safe fn clear_pixels` is E5002 at its declaration, naming `buf.data` and the `unsafe fn clear_pixels(buf: PixelBuf)` line with `@safety` above it.
* The wrapper `fn clear(ref mut bytes: MutSpan[u8])` with `@safety` on `unsafe fn clear_pixels` compiles with no L3015 and no W3012. `clear(ref mut frame.as_mut_span())` is the 0.9.15 spelling, runs, and clears exactly the frame.
* Server case: `safe fn writev(fd: c_int, iov: Span[IoVec], iovcnt: c_int)` is E5002, naming `base` in `iov`'s elements; `unsafe fn writev(fd: c_int, iov: *IoVec, iovcnt: c_int) -> isize` with `@safety` compiles.
* Parser case: the overlay entry `fn cp_load(opts: borrowed one)` is E5002, naming `opts.path`; the `@safety` + `unsafe fn cp_load(opts)` entry compiles.
* Audio case: `@ffi(safe) type mix_fn` is E5002, naming `blk.samples`; check that `fn mixer_get_default()` (a function-pointer result with no words) is accepted as written.
* Game case: `SDL_GetCurrentDisplayMode(displayIndex, mode: out one)`, whose `SDL_DisplayMode` holds `void* driverdata`, compiles and runs unchanged.
* Leaderboard case: the comparator program compiles and prints 10, 20, 30. FFI-52's `register_cb` example (p16-ffi.md 489-508) and tests/conformance/FFI-21/accept_user_data_named_by_its_place.em still compile and pass.
* Network case: `safe fn read_packet(make_buf: extern "C" fn(usize) -> *mut u8, want: usize)` is E5002, naming `make_buf`'s result.
* Asset-loader case: the entry `fn blob_new(len) -> returns_owned(destructor=blob_free) one`, whose `blob` holds `data: *mut u8`, is E5002, naming `blob.data`.
* A result C hands back by value, `safe fn camera_latest_frame() -> FrameInfo` with `pixels: *u8`, compiles, while `safe fn camera_latest_pixels() -> *u8` is still E5002.
* A `ref mut` or `MutSpan` result (exclusive, ODR-078) whose element struct holds a raw pointer is E5002; the same struct as a read-only `ref T` or `Span[T]` result compiles.
* Depth: a raw pointer in a struct field of a struct field, in an enum payload and in a fixed-array element of an argument is reached. An opaque `extern type` behind a contracted pointer is not reached and still compiles.
* An `@ffi(std_function)` type whose parameter struct holds a raw pointer is E5002 at its overlay entry.
* XVI.4's `vkCreateBuffer` (p16-ffi.md 239-241) is now E5002: pCreateInfo's `pNext` and `pQueueFamilyIndices`, and pAllocator's `pUserData`. `pBuffer: out one` is not refused. The example is rewritten once with K46 (J8-14).
* A `Span` of pointer-represented handles (`vkCmdBindVertexBuffers`' `pBuffers` on a 64-bit target) is E5002; check again after K46 settles `@ffi(handle)`.
* No std, example or test declaration is newly refused. Search std, examples and tests for `safe fn` and safe overlay entries whose arguments hold a raw pointer, whose callback types return one, or whose `ref mut`, `MutSpan` or `owns` results hold one. A grep of the main std/src for `safe fn` with a callback returning a raw pointer found only Ember `unsafe fn from_thunks` helpers in worktrees, which are not foreign declarations.

## K20-anyerror-send (P01; J6-16): **unresolved**, 7 agents

Outcome: `decided_by_parameters`. Intent: Load a level file on a worker thread and hand back to the main thread either its length or the reason it failed.
The rule that decides it: THR-8 (with ERR-3's declaration of Error) (p11-concurrency.md (and p13-errors.md) p11 43-51; p13 87-88; also THR-10 p11 175-176, ERR-8 p13 93-95, IFC-3 p05 411-413, p04 676): "THR-8: "A type is **`Send`** when a value of it may be handed over to another thread ... an `owns T` when `T` is; ... `dyn I` when `I` requires it (`interface I: needs Send:`)." ERR-3: "`Error` is the interface of error types: `Display + Debug` with `fn source() ...`" (p04 676: `interface Error: needs (Debug, Display):`, no Send). ERR-8: AnyError is "a `dyn Error` held alone (`owns dyn Error`)" implementing "`From[E]` for every `E: needs Error`". THR-10: "`R` MUST be `Send` (`E7015`, `E7008`)". IFC-3: "a bound brings its parents". Together they decide today, with no one having chosen it, that AnyError is not Send, so the worker is refused at spawn. The compiler cannot tell what an erased error holds; THR-8's own clause for a dyn already says how an erased value is made safe for threads: by what its interface requires. The choice is whether that requirement sits on what an error is, or on the catch-all's conversion."

### Edit: ERR-3 (change, p13-errors.md 88)

**Before:**

> `Display + Debug` with `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8).

**After:**

> `interface Error: needs (Debug, Display, Send, ThreadShared):` with `fn source() -> Option[ref dyn Error]` (default `None`) (§IV.8), so any thread may have and read every error, `dyn Error` and `AnyError` (`[THR-8]`). A type that is not `Send` and `ThreadShared` is `E2040` where it implements `Error`, the message naming what in it is not and the words to write (`[THR-2]`).

Why general: It states the property on what an error is: any thread may have and read it. Every place that names an error then carries it with nothing written: a bound `E: needs Error` or `type Failure: needs Error` (a bound brings its parents, IFC-3 p05 411-413), a field `owns dyn Error`, and AnyError itself (THR-8's existing clause: a dyn is Send when its interface requires it). So every route is covered at once: a thread's result, a channel element, a job's data, a thread_shared holder, an RwLock, a `ref` given to a scoped task. The refusal sits where a type is made an error, the one place its contents are known, and the marker rules decide it for every one-thread part (a shares owner, a weakref link, a sharedref mut link, a Cell) with no list of cases. ERR-1's "any `E: needs Error`", ERR-8's `From` and ERR-9's "any error" stay true as written. The parents are spelled with `needs`, as verdict 110 spells them.

### Edit: THR-8 (change, p11-concurrency.md 51)

**Before:**

> `dyn I` when `I` requires it (`interface I: needs Send:`).

**After:**

> `dyn I` when `I` requires it (`interface I: needs Send:`), as it is `ThreadShared` when `I` requires that (ODR-255).

Why general: THR-9 says ThreadShared is inferred like Send but no rule says it for a dyn. ODR-255 already ruled that a dyn is Send and Sync (now ThreadShared) by what its interface requires. The words now say so for every interface, a program's own (`interface Command: needs (Send, ThreadShared):`) as well as Error. There is no second spelling: the markers go in the interface's `needs`, never after `dyn` with `+`.

### Decision (made by the owner's parameters): Where an error that only one thread may have is refused: where its type is declared an error, where it goes into the catch-all error, or where it leaves the thread

**C++ analogue: should every exception type be required to be safe to hand to and read from any thread, so that a catch-all holder like std::exception_ptr can always cross threads?**

| step | who | what happens | result |
|---|---|---|---|
| 1 | an error type's author | declares an error holding plain values (a code, a text) | allowed: it may go to any thread |
| 2 | worker thread | `fs.read_to_string(path)?` puts the file error into the catch-all error | allowed, nothing written |
| 3 | worker thread | returns `Result[int]` (count or catch-all error) from the thread | allowed: every error may go to any thread |
| 4 | main thread | `n = h.join()?` | gets the count, or passes the error on |
| 5 | an error type's author | declares an error holding a one-thread shared owner | refused when compiling at that declaration, with the line to write |

Today: The worker returning `Result[int]` is refused at `thread.spawn` without the text saying so; it must name a concrete error type instead.

* A: the catch-all error is safe for every thread and checks each error where `?` puts it in; generic error code and error wrappers must then add thread words
* B: keep today: the catch-all stays on its thread; a worker names its own error type, an enum when it can fail two ways
* C: the compiler searches the whole program; one one-thread error put in anywhere makes every thread hand-off of the catch-all fail, far from that line
* D: every error type is safe for every thread (as Swift's errors are), checked where the error type is declared; nothing is written anywhere else
* E: the catch-all may be sent but not read by two threads at once; a shared cache of load results is refused

Recommendation: D: ordinary worker, library and error-wrapping code compile with nothing written; only an error type holding a one-thread part is refused, at its own declaration with the fix (one holding a raw pointer waits on the pending question of vouching for thread sharing).

Chosen: D. By: 1. The three goals at once. B fails 'Python-like', because the ordinary worker grows a concrete error type and an enum per kind. A and E fail it for ordinary library and error-wrapping code, which grows thread words (holes E1-1, E1-2). C and D keep all of it as today. 2. Safety (C4): every option refuses before a one-thread value crosses. A's safety for program-defined erased types also rested on a conversion to `dyn I + Send` that no rule states (hole S1-2); D needs no such conversion. 3. Compile time first (C5): all are decided when compiling. Each adds only verdict 114's existing hand-over check, through a table, when an AnyError crosses (stated in cost). 4. A written word only for meaning (C3): A writes `needs (Send, ThreadShared)` in generic bounds and `+ Send + ThreadShared` on dyn fields, words that say nothing the program does (bookkeeping). B writes a concrete error type. C and D write nothing. 5. Ordinary program no longer or harder (C2): C lets an edit anywhere break a spawn far away. D refuses only an error type holding a one-thread part, at its own line with the words to write. These are the same errors A refused at their `?`, which nearly every error reaches since main returns AnyError, plus the rare one that never leaves its thread. 6. Consistency (C6): D uses THR-8's existing clause (a dyn is Send by what its interface requires) and ODR-255, IFC-3 (a bound brings its parents), verdict 110's `needs` spelling, verdict 61 (the type says which threads may have it), verdict 165 (refused where it is made), no `+` (verdict 109; decisions.md line 141 left `+` after dyn unanswered), and Swift's Error: Sendable. It keeps ERR-1, ERR-8 and ERR-9 true as written. So D.

Option A:

```
## std.core
pub struct AnyError:
    error: owns dyn Error + Send + ThreadShared
    ...
extend[E: needs (Error, Send, ThreadShared)] AnyError implements From[E]:
    ...
## a generic library
[S: needs Source]
pub fn count_lines(src: S) -> Result[int] where S.Failure: needs (Send, ThreadShared):
    text = src.read_all()?
## an error wrapper
struct ImportError:
    cause: owns dyn Error + Send + ThreadShared
```

The worker compiles. A one-thread error is refused at its `?` into AnyError (E2040), even with one thread. Generic code whose error is only `needs Error` must also write `where S.Failure: needs (Send, ThreadShared)`. A wrapper holding `owns dyn Error` must write `owns dyn Error + Send + ThreadShared`, which is a second spelling of the markers, and its `source()` then needs a conversion the spec does not have. ERR-1's 'any `E: needs Error`' and ERR-9's 'any error' become untrue.

Option B:

```
import std.io
fn load(path: StrView) -> Result[int, io.Error]:
    text = fs.read_to_string(path)?
    return Ok(text.len())
```

Today's rules. The `Result[int]` worker is refused at the spawn line (E7015). A worker must name a concrete error type: an @derive(Error) enum with an @from variant per kind when it can fail two ways, converted after join.

Option C:

```
## std unchanged; the compiler collects every error type converted into AnyError anywhere in the program
fn load(path: StrView) -> Result[int]:
    ...
h = thread.spawn(owns fn() => load("level1.cfg"))
```

The worker compiles until some `?` anywhere in the program, even in code that never meets a thread, puts a one-thread error into an AnyError. From then on every spawn, channel or thread_shared holder of AnyError is refused, with a message naming that distant line. This is the whole-program search that verdict 61 turned down for function values.

Option D:

```
## std.core
pub interface Error: needs (Debug, Display, Send, ThreadShared):
    fn source() -> Option[ref dyn Error]:
        return None
## AnyError and its From unchanged; user code unchanged
fn load(path: StrView) -> Result[int]:
    text = fs.read_to_string(path)?
    return Ok(text.len())
h = thread.spawn(owns fn() => load("level1.cfg"))
```

The worker compiles. Generic error code (`needs Error`) and wrappers holding `owns dyn Error` compile with nothing written, and every rule that says 'any error' stays true. An error type holding a one-thread part (a shares owner, a weakref link to a one-thread value, a sharedref mut link, a Cell) is refused where it implements Error (E2040, with the words to write), even if it never leaves its thread. One holding a raw pointer is refused until its author may vouch for ThreadShared, which is the pending vouching question (P04, J7-06); A refused the same error at its `?`. This is Swift's choice: its Error refines Sendable.

Option E:

```
pub struct AnyError:
    error: owns dyn Error + Send
extend[E: needs (Error, Send)] AnyError implements From[E]:
```

The worker compiles. AnyError is Send but not ThreadShared, so `assets = thread_shared Assets()` holding `RwLock[Array[Result[Texture]]]` is refused at its line (verdict 165), and a `ref` to an AnyError cannot go to scoped tasks. It has A's costs for Send: generic bounds and dyn fields must name it.

Readability: The after is shorter (no `import std.io`, no concrete error type) and reads like the one-thread program. With two kinds of failure it also saves a whole error enum. It adds no word anywhere. Generic error code (`type Failure: needs Error`) and error wrappers holding `owns dyn Error` stay exactly as today, where option A made them grow `needs (Send, ThreadShared)` and `+ Send + ThreadShared`. The cost falls only on an error type holding a one-thread part, which is refused at its own declaration with the words to write.
Cost: Compile time: Send and ThreadShared are inferred, and are checked once, where a type implements Error. A `?` into AnyError is checked as today. Run time: AnyError keeps its layout (the boxed error's data and table pointers, its identity, the context list) and its one allocation on the failure path (ERR-11). A `?` into it costs what it costs today, and the success path costs nothing, as in C. Handing an AnyError to another thread runs verdict 114's hand-over check on what its error owns (THR-2 p11 119-121) through the error's table: one indirect call per AnyError handed over, then one compare for each object it owns whose class has weakref links. Nothing runs in a program where no class has weakref links. C pays nothing to pass an error pointer to another thread. To measure: AnyError's size and failure-path time against today's build, and the time of that hand-over check.

Final check of the last breakers' holes: 3 confirmed real, 0 refuted, 0 already in today's specification (only the confirmed count)

### Holes left (3)

* **regression**, medium: A common kind of generic library error hands the caller's value back. Rust has `SendError<T>`, `TrySendError<T>` and arrayvec's `CapacityError<T>`; std's own SendError[T] holds the value it could not send. Under the edit such an error needs thread words where it implements Error, and again in every pub generic function that uses `?` on it. The one-thread game whose pool holds `shares` owners is refused at its call, far from any error declaration. These are A's costs: E1-1's bound, and the refusal at the use rather than at the declaration. The edit says D removes them. Its readability verdict says 'It adds no word anywhere'. Decision parameter 4 says 'C and D write nothing'. why_general says 'every place that names an error then carries it with nothing written'. The cost section says the refusal is 'at its own declaration ... the one place its contents are known', but a generic error's contents are not known there. Under A this library's extend line needed nothing, so here D writes more words than A, not fewer. The owner would compare A and D on a wrong account.

```
from std.fmt import Formatter, FmtError

## a game engine's fixed-size pool: a full pool hands the value back to the caller
[T]
pub struct PoolFull:
    pub rejected: T

extend[T] PoolFull[T] implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("the pool is full")

extend[T] PoolFull[T] implements Debug:
    fn fmt_debug(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str("PoolFull")

extend[T] PoolFull[T] implements Error:
    pass

[T]
pub struct Pool:
    items: Array[T]
    cap: int

extend[T] Pool[T]:
    pub fn insert(owns v: T) -> Result[void, PoolFull[T]]:
        if this.items.len() == this.cap:
            return Err(PoolFull(rejected=v))
        this.items.push(owns v)
        return Ok(())

[T]
pub fn fill(ref mut pool: Pool[T], owns items: Array[T]) -> Result[void]:
    for x in owns items:
        pool.insert(owns x)?            # PoolFull[T] -> AnyError
    return Ok(())

class Enemy:
    hp: int = 10

fn main() -> Result[void]:
    pool: Pool[shares Enemy] = Pool(items=[], cap=2)
    wave: Array[shares Enemy] = [shares Enemy(), shares Enemy(), shares Enemy()]
    fill(ref mut pool, owns wave)?      # a one-thread game
    return Ok(())
```

* **undecided**, medium: The edited words leave an error type, here a generic one, undecided between a refusal and a silent change in what the program prints. Under reading (i), a one-thread program that compiles today still compiles, but its report loses the cause of the failure with no message. That is code that compiles but does something not meant (principle 7, checklist 1 and 8). Under reading (ii), a generic error type that holds the caller's value cannot serve one-thread callers. The edit's to_verify defers the choice, but the outcome of an ordinary program depends on it.

```
import std.fs
import std.io

class Asset:
    name: String

## an asset library's error: a failed read, or an asset the loader refused, handed back to the caller
@derive(Error)
enum LoadError[T]:
    @error("cannot read the asset list")
    @from
    Io(io.Error)
    @error("asset refused")
    Refused(item: T)

fn load_list(path: StrView) -> Result[int, LoadError[shares Asset]]:
    text = fs.read_to_string(path)?          # io.Error -> LoadError through @from
    return Ok(text.lines().count())

fn main() -> Result[void, LoadError[shares Asset]]:
    println(load_list("assets.txt")?)
    return Ok(())
```

* **cross_rule_clash**, medium: A conformance test for J6-16 can be written to either verdict, and so can a test for every sibling: the game's `thread_shared Assets()`, the generic library's `S.Failure` (IFC-3 brings whichever parents the definition has), and the parser's E2040. The first round counted the disagreement of ERR-1 and ERR-9 with the edited ERR-8 as real holes (S1-1, E1-3) for exactly this reason. The repair moved the disagreement to IV.8 instead of removing it. The edit's notes name the change to p04 676 and core.em 80 as a follow-up for the lead, but nothing in the edited words makes ERR-3 win over the definition it cites.

```
import std.fs
import std.thread

fn load(path: StrView) -> Result[int]:            # fails with any error: AnyError
    text = fs.read_to_string(path)?               # io.Error -> AnyError
    return Ok(text.len())

fn main() -> Result[void]:
    h = thread.spawn(owns fn() => load("level1.cfg"))   # the result, Result[int, AnyError], must be Send
    n = h.join()?
    println(n)
    return Ok(())
```

New gaps found on the way (not this edit's): 4

Tests rejected by the script, not written (3):
* generic_error_int_arg: the expected output is a description, not the exact text: true if the derive implements Error per instance. Under the other reading of hole 1, E2040
* j6_16_worker_returns_anyerror: the expected output is a description, not the exact text: With the edit: compiles and prints the length of level1.cfg; when the file is missing, std
* loaderror_generic_derive: the expected output is a description, not the exact text: To be decided (hole 2). Either E2040 at `@derive(Error)` giving the bound, or, with assets

To verify on the compiler:
* The compiler today on the J6-16 program: the spec says it is refused at the spawn line (E7015/E7008); check whether it is refused, or compiles because `R: Send` is not checked yet.
* After std/src/core.em line 80 becomes `pub interface Error: needs (Debug, Display, Send, ThreadShared):` (lines 93 and 97 unchanged): J6-16 compiles and prints the length.
* `dyn Error` is inferred Send and ThreadShared from Error's parents (THR-8's words), so AnyError (`Box[dyn Error]`, `u128`, `Array[String]`) is both. The game sibling compiles and prints 1, with `thread_shared Assets()` accepted. The server sibling prints `/index` then `error: empty request path`.
* The generic-library sibling compiles unchanged and prints 3: `type Failure: needs Error` brings Send and ThreadShared inside the body. The importer sibling compiles unchanged, `source()` returns `Some(ref this.cause)`, and main prints the cause chain.
* The parser sibling: E2040 at `@derive(Error)` on ParseError, naming `file` (a shares owner). The suggested `path: String` (built with `path=copy of file.path`) compiles, and so does the thread_shared file.
* Every std error type still implements Error: io.Error, FmtError, Missing and Message (core.em 222, 228), string.em's ParseError, SerError, CppError, RangeError. SendError[T] implements Error for T that are Send and ThreadShared, and `tx.send(...).expect(...)` (Display only) is unaffected for every T.
* `@derive(Error)` on a generic type (`enum LoadError[T]`) implements Error for the instances whose T is Send and ThreadShared, or is refused with a message giving the bound; decide which and check the message compiles.
* Regression scan: every error type in std, the spec's examples and the test suites, listing those now refused (expected: none in std; any holding shares, weakref, sharedref mut, Cell or raw-pointer fields).
* A type that is Send but not ThreadShared (a vouched raw pointer) implementing Error is E2040 at its `extend ... implements Error` line, with words that compile.
* Verdict 114's hand-over check reaches objects owned by the error inside an AnyError returned from a thread, through the error's table.
* To measure: AnyError's size and the failure-path time of a `?` into it, unchanged against today's build; and the time of the hand-over check of an AnyError in a program with a weakref'd class.

## K21-cpp-overloads (P07; J8-04): **unresolved**, 7 agents

Outcome: `decided_by_parameters`. Intent: Build a C++ renderer with its (width, height) constructor, then draw a mesh and a sprite through its two `draw` overloads, running the functions the C++ header means.
The rule that decides it: FFI-39b (p21-annex-c.md 128-130): "`super.init(…)` selects the C++ base constructor by arity and argument types and runs it at `super.init(…)` (`[CLS-11]`)"

### Edit: TYP-26 (add, p04-types.md 867-868)

**Before:**

> There is no overloading: two functions of one name in one scope are `E1030`, except operator-interface implementations and `extend` blocks for different types.

**After:**

> A C++ name of several functions imports as one name (`[FFI-39b]`): the header, not the program, declares them.

Why general: Says why the no-overloading rule does not reach a header (principle 6). Refusing a header's overloads would refuse correct programs over code they did not write and cannot change. It covers every C++ name of several functions (namespace functions, methods, static functions, constructors) without naming any of them.

### Edit: FFI-39b (add, p21-annex-c.md 128-130)

**Before:**

> `super.init(…)` selects the C++ base constructor by arity and argument types and runs it at `super.init(…)` (`[CLS-11]`); a base with no default constructor and no declared `init` is `E5058`.

**After:**

> Each use of a C++ name of several functions (as `[FFI-30]` counts them) stands for the one it fits, ranked as `[TYP-24]` ranks instances, an untyped literal counting as C++ types it (`c_int`, `f64`) and an object as `ref` through a name that only reads it or in an entry with `@ffi(const)`, else `ref mut`: a call (`new`, `super.init(…)`) by its object and its arguments' number, names, types and words; an `override fn` by its parameters; a function value by its expected callable type; an overlay entry by what it writes. What the code around it permits is checked after, on the one chosen. None, or several fitting equally, is `E2070` (an entry `E5010`), naming each.

Why general: One property for every use of such a name, not just calls. A use stands for the one function it fits, ranked the way Ember already ranks an interface's instances (exact type before coercion). Literals count as C++ types them, and the object counts as C++ counts its implied object, so where C++ picks a function Ember picks the same one or refuses. Nothing more is needed for constructors, `super.init`, methods, namespace functions, overrides, function values and overlay entries. A redeclared function counts once. What the code around a use permits (unsafe code, effects, values handed over, threads, views) is checked on the chosen function, so a line runs the same function wherever it stands. A use that fits nothing, or ties, is refused with the candidates named.

### Edit: GRM-35 (add, p16-ffi.md 266-267)

**Before:**

> In an entry marked `@ffi(macro_fn)` (`[FFI-6b]`) a parameter's and the result's words are one Ember type each (`IDENT ":" type`, `"->" type`), since a macro has no C declaration to read (ODR-147).

**After:**

> An entry may write each parameter's and the result's word and type as imported (`owns value: Texture`), checked, before its contract words; `[FFI-30c]` and `[FFI-55]` count entries by function.

Why general: The single entry form gets an optional typed parameter. It is Ember's own spelling of a parameter, already written in `@ffi(macro_fn)` entries. With FFI-39b's 'an overlay entry by what it writes', the type is needed exactly where parameter names cannot tell functions apart; elsewhere it may be written and is checked. Every overlay rule that counts entries (`override`, a second entry, the `ember.overlays` record) then counts per function, for every overloaded name.

### Decision (made by the owner's parameters): How a C++ name declared several times imports, and which function each use stands for

**A C++ class with two constructors and two `draw(...)` overloads: what Ember calls them, and which one `r.draw(mesh)` runs.**

| step | who | what happens | result |
|---|---|---|---|
| 1 | importer | reads `Renderer()`, `Renderer(int, int)`, `draw(const Mesh&)`, `draw(const Sprite&)` | one `new` and one `draw` on `Renderer` |
| 2 | program | writes `Renderer.new(1280, 720)` and `r.draw(mesh)` | no new word |
| 3 | compiler | keeps the functions the call's own words fit (count, names, types, `owns` / `ref mut`, the object) | `Renderer(int, int)`, `draw(const Mesh&)` |
| 4 | compiler | takes the exact fit; none, or still two, is an error naming each; then checks `unsafe` and the rest on that one | the same function wherever the line stands |
| 5 | program runs | a direct call of that function | the same cost as the C++ call |

Today: Nothing says. Read literally, two functions of one name are an error, and only `super.init` picks a C++ constructor by its arguments.

* A: one name per C++ name; each use picks by its own words, as C++ does and as `super.init` already does
* B: one name per overload, built from its types (`draw_Mesh`, `new_int_int`)
* C: overloads skipped unless an overlay renames each one
* D: as A, but a call picks among the functions it could compile as where it stands, so moving it into an `unsafe:` block can change which one runs

Recommendation: A: the C++ program's own calls work unchanged, one line always runs one function, and an unclear call is refused with the candidates named.

Chosen: A. By: 1. The three goals: A keeps C++'s names, adds no word and costs nothing while running. B and C add names the program must write or keep. 2. Safety: A runs one function per line wherever the line stands, and refuses a use that no function fits, or that two fit equally. D lets the surrounding code change what runs, with no diagnostic. 3. Compile time first: A is decided when compiling, as a direct call of the chosen thunk. 4. A written word only for meaning: B's type-built names and C's renames are bookkeeping. 5. The ordinary program: A is the C++ program's own lines. B is longer and breaks when a type is renamed. C needs an overlay line per overload before anything works. 6. Consistency: FFI-39b already picks `super.init`'s constructor by arity and argument types. TYP-24 already chooses among one name's interface instances by a call's arguments. FFI-48 refuses only overloads no call can tell apart. FFI-44 imports classes automatically, and FFI-40 and FFI-41 keep C++'s names. C++ chooses by the call alone, never by the code around it.

Option A:

```
r = eng.gfx.Renderer.new(1280, 720)   # Renderer(int, int)
r.draw(mesh)                           # draw(const Mesh&)
r.draw(hud)                            # draw(const Sprite&)
```

Compiles. There is one Ember name per C++ name. Each use stands for the function its own words fit: a call by its object and arguments, an override by its parameters, a function value by its expected type. Exact types are taken first. What the code around the use permits is checked afterwards, on that function. None, or two fitting equally, is an error naming each C++ declaration.

Option B:

```
r = eng.gfx.Renderer.new_int_int(1280, 720)
r.draw_Mesh(mesh)
r.draw_Sprite(hud)
```

Compiles. Each overload is its own Ember function, named from C++ types the program never wrote, and the names change when the header renames a type.

Option C:

```
unsafe overlay cpp "renderer.hpp":
    class Renderer:
        rename new(width, height) as sized
        rename draw(mesh) as draw_mesh
        rename draw(sprite) as draw_sprite
```

Without the overlay every overload is skipped, and `r.draw(mesh)` gives a 'skipped: overloaded' diagnostic. With it, `r.draw_mesh(mesh)` compiles. Today's `rename` grammar takes only a name, so the grammar would have to change too.

Option D:

```
ring.push(pcm)            # push(int32_t): the only one with an overlay entry
# SAFETY: push(int16_t) only stores the sample in the ring's own buffer
unsafe:
    ring.push(pcm)        # push(int16_t): the exact fit
```

Compiles, but the same line runs two different C++ functions. A call chooses among the functions it could compile as where it stands, so moving it into or out of an `unsafe:` block, or into a `@noalloc` function, silently changes which one runs.

### Decision (made by the owner's parameters): How an overlay entry names one of several C++ functions alike in name, parameter count and parameter names

**Two C++ overloads with the same parameter names, `set(const char* name, float value)` and `set(const char* name, int value)`: how an overlay line says which one it vouches for.**

| step | who | what happens | result |
|---|---|---|---|
| 1 | overlay writer | writes `fn set(name: borrowed nul_terminated, value)` | it fits both overloads |
| 2 | compiler | refuses it, naming both C++ declarations | error |
| 3 | overlay writer | writes `fn set(name: *c_char borrowed nul_terminated, value: f32)`, and the same line with `value: c_int` | each line names one overload |
| 4 | compiler | checks each line against its own overload, as for any entry | both overloads safe to call |

Today: Overlay lines are matched by name, count and parameter names only, so such a pair cannot be told apart, and a second line for one name is an error.

* a: one line covers every overload it fits
* b: the line may write the parameters' Ember types, as Ember writes a function's first line, and must where the names do not tell
* c: refuse, with no way to vouch for either overload
* d: write the C++ signature in an attribute, `@ffi(cpp="set(const char*, float)")`

Recommendation: b: Ember's own spelling of a parameter, needed only where the names do not already tell the overloads apart.

Chosen: b. By: 1. The three goals: b keeps the overlay readable as Ember and costs nothing while running. 2. Safety: a lets one line vouch for a function its writer never named (verdict 131's problem). b, c and d refuse an entry that fits several functions. 3. Compile time first: all four are decided when compiling. 4. A written word only for meaning: which function a line vouches for is meaning. Under b the type is needed only where the names do not tell; elsewhere it is optional and checked. c makes it impossible to write, an exemption that exists only because syntax is missing (ODR 13's lesson). 5. The ordinary overlay: unchanged under b; d adds an attribute line per overload. 6. Consistency: b is Ember's own spelling of a parameter (`value: f32`, `owns value: Texture`), optional as a variable's type is, and overlays already write types in `@ffi(macro_fn)` entries (checklist 11). d is a second spelling. On 2026-10-29 the lead removed the separate typed member line ('one way to write an overlay line'); b keeps one entry form, whose parameters may carry a type.

Option a:

```
unsafe overlay cpp "shader.hpp":
    class Shader:
        fn set(name: borrowed nul_terminated, value)   # covers both `set` overloads
```

Compiles. One line makes both overloads safe to call, including one its writer may never have looked at, and both must take the same contract words.

Option b:

```
unsafe overlay cpp "shader.hpp":
    class Shader:
        fn set(name: *c_char borrowed nul_terminated, value: f32)
        fn set(name: *c_char borrowed nul_terminated, value: c_int)
```

Compiles. An entry may write a parameter's type and word as the import maps them, checked against the header. Each of these lines therefore names one function and is checked against it. The untyped line fits both and is E5010, naming both. Entries whose names already differ are written as today.

Option c:

```
unsafe overlay cpp "shader.hpp":
    class Shader:
        fn set(name: borrowed nul_terminated, value)   # E5010: fits two functions
```

Refused, and no line can name either overload. Both stay unsafe to call, with no place to write their `@safety`.

Option d:

```
unsafe overlay cpp "shader.hpp":
    class Shader:
        @ffi(cpp="set(const char*, float)")
        fn set(name: borrowed nul_terminated, value)
        @ffi(cpp="set(const char*, int)")
        fn set(name: borrowed nul_terminated, value)
```

Compiles. Each overload is named by C++ text in a string: a second spelling, in another language, of what a parameter list already says.

### Decision (made by the owner's parameters): Which overload an untyped number literal picks

**C++ `set(const char* name, int value)` and `set(const char* name, float value)`: which one `s.set(c"mode", 2)` runs.**

| step | who | what happens | result |
|---|---|---|---|
| 1 | program | writes `s.set(c"mode", 2)` | a plain `2` |
| 2 | compiler | counts `2` as C++ counts it, an `int` | |
| 3 | compiler | `set(…, int)` takes it as it is, `set(…, float)` only by conversion | `set(const char*, int)` |
| 4 | compiler | where C++ itself cannot choose (`value(int64_t)` / `value(double)` with `7`), an error naming both | `7 as i64` picks one |

Today: Nothing says. Read as Ember's other overload choice, `2` counts as Ember's 64-bit `int`, which neither function takes, so the call is an error.

* i: count a literal as Ember's default (64-bit `int`, `f64`): `s.set(c"mode", 2)` is an error; write `2 as c_int`
* ii: a whole-number literal prefers any whole-number type: `set` works, but `put(int)` / `put(long)` with `5`, and `scale(float)` / `scale(double)` with `2.0`, are errors
* iii: count a literal as C++ does (`int`, `double`): Ember runs what C++ runs, and refuses where C++ refuses

Recommendation: iii: the C++ lines work unchanged and never run a different function from the one C++ would.

Chosen: iii. By: 1. The three goals: iii keeps the C++ program's lines with no cast and costs nothing while running. 2. Safety: all three are decided when compiling and refuse rather than guess. iii never runs a different function from C++, because Ember's exact fit is C++'s and Ember's conversions are fewer. 3. Compile time first: all three. 4. A written word only for meaning: i's `as c_int` on every literal is bookkeeping, and so are the casts ii needs. 5. The ordinary program: under iii it is the C++ line. i and ii refuse common calls (uniforms, int/long, float/double). 6. Consistency: when choosing among one name's instances, the nearest rule (TYP-24) already counts a literal as its default type. iii does exactly that, with the defaults of the language the header is written in (`int`, `double`).

Option i:

```
s.set(c"mode", 2)            # error: `2` counts as Ember's `int` (i64); both `set` reach it by conversion
s.set(c"mode", 2 as c_int)   # what the program must write
```

Refused, with `as c_int` to write on every literal call of an int/float overload pair; `super.init(16, 8)` too.

Option ii:

```
s.set(c"mode", 2)            # set(const char*, int): a whole-number literal prefers whole-number types
out.put(5)                   # error: put(int) and put(long) both take whole numbers
m.scale(2.0)                 # error: scale(float) and scale(double) both take floats
```

Compiles for `set`. Refuses the int/long and float/double pairs, which C++ calls without trouble.

Option iii:

```
s.set(c"mode", 2)            # set(const char*, int): `2` counts as C++'s `int`
out.put(5)                   # put(int)
m.scale(2.0)                 # scale(double): `2.0` is C++'s `double`
w.value(7)                   # error, as in C++: value(int64_t) and value(double) both convert
```

Compiles wherever C++ picks one, running the same function. Refused, naming both, where C++ is ambiguous too.

### Decision (made by the owner's parameters): Which of a const and a non-const C++ method a call runs

**C++ `const Entry& at(size_t) const` and `Entry& at(size_t)`: which one `cfg.at(0)` runs.**

| step | who | what happens | result |
|---|---|---|---|
| 1 | program | writes `cfg.at(0).port = 8080` in `main`, where `cfg` may be changed | no new word |
| 2 | compiler | the non-const `at` fits `cfg` as it is, the const one only as a read-only view | `Entry& at(size_t)` |
| 3 | compiler | in `port_of(cfg: Config)`, `cfg` only reads, so only the const `at` fits | `const Entry& at(size_t) const` |
| 4 | program runs | prints `8080` | as in C++ |

Today: Nothing says. Read literally, two functions of one name are an error.

* a: both fit a changeable name equally: an error, and the non-const `at` can never be called
* b: as in C++: a changeable name picks the non-const `at`, a read-only one the const `at`
* c: the const `at` unless the result is changed, as Rust picks between its two indexing interfaces

Recommendation: b: what C++ runs, decided by the call alone, and every function in the pair can be called.

Chosen: b. By: 1. The three goals: b keeps C++'s meaning with no word and no run-time cost. 2. Safety: b and c are safe, but c runs a different function from C++ for the same call; a refuses. 3. Compile time first: all three. 4. A written word only for meaning: none of them needs one. a needs a helper function to reach the const `at` through a changeable name, and leaves the other `at` unreachable, an exemption that exists only because syntax is missing (ODR 13's lesson). 5. The ordinary program: b compiles the C++ lines; a refuses them. 6. Consistency: the object counts as one more argument under the nearest rule (exact before coercion; a read-only view of a changeable place is a coercion, TYP-5 rule 7), as C++ counts its implied object. An overlay entry's `@ffi(const)` (verdict 132) then names which of the pair it describes, with no new word. c would choose by the code around the call, which the first decision excludes. K47 (J8-05) keeps its own question, the overlay name of `operator[]`, whose pair goes through Index/IndexMut (FFI-42), not this selection.

Option a:

```
cfg.at(0).port = 8080      # error: both `at` fit a changeable `cfg` equally
```

Refused through every changeable name, which is the ordinary local. No line can ever call `Entry& at`.

Option b:

```
cfg.at(0).port = 8080      # Entry& at(size_t): `cfg` may be changed
# in fn port_of(cfg: net.srv.Config):
return cfg.at(0).port      # const Entry& at(size_t) const: `cfg` only reads
```

Compiles and runs what C++ runs. The non-const result is a changing view of `cfg` while it is used, as C++'s `Entry&` is, so two such results held at once through one changeable name are refused.

Option c:

```
e = cfg.at(0)              # const at: the result is only read
cfg.at(0).port = 8080      # Entry& at: the result is changed
```

Compiles more Ember programs, but which C++ function a call runs depends on the lines after it, and can differ from C++. For example, a copy-on-write string's non-const accessor copies and the const one does not.

Readability: No longer and no harder. The ordinary program is shorter than today's shim. Extra words appear only on the rare overlay entry whose parameter names cannot say which function it vouches for.
Cost: None at run time: it is decided when compiling. Each call is a direct call of the chosen function's thunk, just as a C++ call resolves its overload when compiling. Per call that is the same single thunk call as any imported C++ function, inlined across the boundary under the project's LTO. There is no table and no check while running. Compile time adds one fit test per candidate function at each use of an overloaded name (to measure).

Final check of the last breakers' holes: 9 confirmed real, 0 refuted, 1 already in today's specification (only the confirmed count)

### Holes left (9)

* **unsafe_allowed**, high: `send_focus` calls `h.on(FocusEvent())`. The slot runs `KeyLog.on` with a FocusEvent taken as a KeyEvent, and `e.code()`, which the overlay vouches safe and which is true for a real KeyEvent, reads `code_` at offset 8 of an 8-byte object. In C++ terms this is an unchecked `static_cast` downcast, a read past the object, and it happens in safe Ember code: both `unsafe:` notes are true. C++ itself refuses this class ('marked override but does not override'). The scalar case goes wrong the same way: `override fn on(timer_id: i16)` fits `on(int)` by widening, so C++'s 70000 reaches an `i16` truncated, while the safe direction, `override fn on(timer_id: int)`, fits nothing. For an override the coercion runs backwards, because C++'s argument flows into the Ember parameter.

```
// input.hpp (C++)
// namespace ui {
// class Event {
// public:
//     Event() noexcept;
//     virtual ~Event() noexcept;
// };
// class KeyEvent : public Event {
// public:
//     explicit KeyEvent(int code) noexcept : code_(code) {}
//     int code() const noexcept { return code_; }
// private:
//     int code_;
// };
// class FocusEvent : public Event {        // no field of its own: 8 bytes; a KeyEvent is 16
// public:
//     FocusEvent() noexcept;
// };
// class Handler {
// public:
//     Handler() noexcept;
//     virtual ~Handler() noexcept;
//     virtual void on(const Event& e) noexcept {}
//     virtual void on(int timer_id) noexcept {}
// };
// void send_key(Handler& h, int code) noexcept;   // calls h.on(KeyEvent(code))
// void send_focus(Handler& h) noexcept;           // calls h.on(FocusEvent())
// }
# overlays/input.em
unsafe overlay cpp "input.hpp":
    class KeyEvent:
        @ffi(const)
        fn code()

# src/main.em
from std.ffi import c_int
import cpp "input.hpp" with (project="ui", overlay="overlays/input.em") as gui

class KeyLog(gui.ui.Handler):
    last: c_int = 0

    fn init():
        # SAFETY: the base constructor sets no state Ember reads
        unsafe:
            super.init()

    override fn on(e: gui.ui.KeyEvent):
        this.last = e.code()

fn main():
    log = KeyLog()
    # SAFETY: the senders call `on` on the handler during the call only
    unsafe:
        gui.ui.send_key(ref mut log, 27)
        gui.ui.send_focus(ref mut log)
    println(log.last)
```

* **needless_refusal**, medium: C++ ranks argument by argument: set(int, int) beats set(int, float) on the second argument and ties on the first, and the non-const `light` beats the const one on the object. Ember refuses both ordinary lines. The same refusal hits every index call of a const/non-const accessor with a literal or an `int` index, and every pair where one shared argument needs a widening. The way out is a cast on each argument (`slot as c_int`, `0 as usize`), the bookkeeping decisions 1 and 4 say the edit removes. S1-3's own program (`cfg.at(0).port = 8080`) is still refused, so S1-3 and E1-10 are not repaired. The same cause ties a nearer base against a farther one (`draw(const Shape&)` / `draw(const Circle&)` with a FilledCircle) and a promotion against a conversion (`put(int)` / `put(long)` with an `i16`), both of which C++ picks.

```
// scene.hpp (C++)
// namespace gfx {
// struct Light { float intensity; };
// class Scene {
// public:
//     Scene() noexcept : lights_{}, counts_{} {}
//     void set(int slot, int value) noexcept { counts_[slot & 3] = value; }
//     void set(int slot, float value) noexcept { lights_[slot & 3].intensity = value; }
//     const Light& light(size_t i) const noexcept { return lights_[i & 3]; }
//     Light& light(size_t i) noexcept { return lights_[i & 3]; }
// private:
//     Light lights_[4];
//     int counts_[4];
// };
// }
# overlays/scene.em
unsafe overlay cpp "scene.hpp":
    class Scene:
        fn new()
        fn set(slot: c_int, value: c_int)
        fn set(slot: c_int, value: f32)
        @ffi(const)
        fn light(i)
        fn light(i)

# src/main.em
import cpp "scene.hpp" with (project="engine", overlay="overlays/scene.em") as gfx

fn main():
    scene = gfx.gfx.Scene.new()
    for slot in 0..4:
        scene.set(slot, 4)
    scene.light(0).intensity = 0.5
    println(scene.light(0).intensity)
```

* **undecided**, medium: Under the import's reading no line can call either `at`, unsafe code included, and no overlay line can vouch for either. That is the exemption that exists only because a word is missing (checklist 12). Bodies in a .cpp are the usual case for a non-template class. A `mutable` member (a cache, a mutex) in a const method is common too, and FFI-40a reads it as changing as well. E1-7's program (`visit` declared without a body) fails the same way, so E1-7 is not repaired. The answer's config sibling works only because its bodies are in the header ('The importer reads that the const `at` changes no field').

```
// config.hpp (C++): bodies in config.cpp, as for most non-template classes
// namespace srv {
// struct Entry { int port; };
// class Config {
// public:
//     Config() noexcept;
//     const Entry& at(size_t i) const noexcept;   // defined in config.cpp: returns entries_[i & 3]
//     Entry& at(size_t i) noexcept;               // defined in config.cpp: returns entries_[i & 3]
// private:
//     Entry entries_[4];
// };
// }
# overlays/config.em
unsafe overlay cpp "config.hpp":
    class Config:
        fn new()
        @ffi(const)
        fn at(i)
        fn at(i)

# src/main.em
from std.ffi import c_int
import cpp "config.hpp" with (project="server", overlay="overlays/config.em") as net

fn port_of(cfg: net.srv.Config, i: usize) -> c_int:
    return cfg.at(i).port

fn main():
    cfg = net.srv.Config.new()
    first: usize = 0
    cfg.at(first).port = 8080
    println(port_of(cfg, first))
```

* **undecided**, medium: Under the first answer `show(s: Server)` is E3023 ('`port` may change its object'). Under the second it compiles and prints 0. The same program compiles on one implementation and not on another, and runs a different C++ function, which matters wherever the pair differs (copy-on-write detaching, or a const accessor that is thread-safe beside a non-const one that is not). C++ decides by the method's own `const`, which Ember does not write. A lambda using a real variable (`fn() => println(cfg.at(i).port)`, CLO-2) is circular the same way. Wrapping a C++ object in an Ember class field is an ordinary pattern.

```
// config.hpp (C++)
// namespace srv {
// struct Entry { int port; };
// class Config {
// public:
//     Config() noexcept : entries_{} {}
//     const Entry& at(size_t i) const noexcept { return entries_[i & 3]; }
//     Entry& at(size_t i) noexcept { return entries_[i & 3]; }
// private:
//     Entry entries_[4];
// };
// }
# overlays/config.em
unsafe overlay cpp "config.hpp":
    class Config:
        fn new()
        @ffi(const)
        fn at(i)
        fn at(i)

# src/main.em
from std.ffi import c_int
import cpp "config.hpp" with (project="server", overlay="overlays/config.em") as net

class Server:
    cfg: net.srv.Config

    fn init():
        this.cfg = net.srv.Config.new()

    fn port(i: usize) -> c_int:
        return this.cfg.at(i).port

fn show(s: Server, i: usize):
    println(s.port(i))

fn main():
    s = Server()
    show(s, 0)
```

* **regression**, high: A promise written and checked for `draw(const Mesh&)` silently vouches for a function nobody reviewed: `count=100000` reads past the renderer's table in safe code (in C++ terms, a buffer over-read). This happens for every C++ entry whose function's parameter types change, overloaded or not, which is exactly the header update verdict 121's record exists to catch. The edit leaves the key to tooling ('the key format is the lead's tooling'), but the key decides whether the check happens.

```
// renderer.hpp (C++), version 2 of the engine SDK
// namespace gfx {
// struct Batch { int first; int count; };
// class Renderer {
// public:
//     Renderer() noexcept;
//     ~Renderer() noexcept;
//     void draw(const Batch& mesh) noexcept;   // draws meshes first .. first+count of its table, unchecked
// };
// }
// (version 1, which the overlay was checked against and ember.overlays recorded:
//  `struct Mesh { int vertices; };` and `void draw(const Mesh& mesh) noexcept;`, reading only the mesh given)
# overlays/renderer.em
unsafe overlay cpp "renderer.hpp":
    class Renderer:
        fn new()
        fn draw(mesh)

# src/main.em
import cpp "renderer.hpp" with (project="engine", overlay="overlays/renderer.em") as eng

fn main():
    r = eng.gfx.Renderer.new()
    r.draw(eng.gfx.Batch(first=0, count=100000))
```

* **undecided**, medium: Under the import reading, an ordinary C++ class whose const accessor is defined out of line cannot be called through any name, and no overlay can name either function. The same holds for a class with a `mutable` member (a mutex), or whose const body calls something the importer cannot follow. The edit's verdict-132 claim, '`@ffi(const) fn visit(ref mut v: Visitor)` describes the const `visit`', fails for its own E1-7 header, whose `void visit(Visitor& v) const noexcept;` has no body. C++ runs `Entry& at` in `main`, runs the const `at` in `port_of`, and prints 8080.

```
// config.hpp (C++): the methods are defined in config.cpp
// namespace srv {
// struct Entry { int port; };
// class Config {
// public:
//     Config() noexcept;
//     ~Config() noexcept;
//     const Entry& at(size_t i) const noexcept;   // aborts unless i < 4
//     Entry& at(size_t i) noexcept;               // aborts unless i < 4
// private:
//     Entry* entries_;
// };
// }
# overlays/config.em
unsafe overlay cpp "config.hpp":
    class Config:
        fn new()
        @ffi(const)
        fn at(i)
        fn at(i)

# src/main.em
import cpp "config.hpp" with (project="server", overlay="overlays/config.em") as net

fn port_of(cfg: net.srv.Config) -> c_int:
    return cfg.at(0).port

fn main():
    cfg = net.srv.Config.new()
    cfg.at(0).port = 8080
    println(port_of(cfg))
```

* **needless_refusal**, medium: In C++, `int heading() const { return transform().angle; }` runs the const one and prints 90. An Ember class over an engine's C++ base is FFI-39's main use, and such bases commonly pair const and non-const accessors. Under the edit, a method that only reads is refused through every read-only name, and no word can say it only reads (verdict 86 removed `reads`). The way out is a free helper that takes the component as a plain parameter, which is the reason decision 4 gives for rejecting option a.

```
// scene.hpp (C++)
// namespace eng {
// struct Transform { int x; int y; int angle; };
// class Component {
// public:
//     Component() noexcept;
//     virtual ~Component() noexcept;
//     const Transform& transform() const noexcept { return transform_; }
//     Transform& transform() noexcept { return transform_; }
// private:
//     Transform transform_{0, 0, 90};
// };
// }
import cpp "scene.hpp" with (project="engine") as core

class Spinner(core.eng.Component):
    fn init():
        # SAFETY: the base constructor only sets its own transform
        unsafe:
            super.init()

    fn heading() -> c_int:
        # SAFETY: `transform` only gives a view of the component's own transform
        unsafe:
            return this.transform().angle

fn show(s: Spinner):
    println(s.heading())

fn main():
    s = Spinner()
    show(s)
```

* **needless_refusal**, medium: C++ runs `show(int)` (an `unsigned char` to `int` is a promotion, which beats the conversions to `long long` and `double`) and prints `int 3`. Under the edit the line needs `lives as c_int`. That is the bookkeeping cast decision iii removed for literals, back again for every narrow number (game counters in `u8`, audio samples in `i16`) passed to the int/long/double overload sets common in C++ APIs. C++ also ranks argument by argument; the edit ranks the whole call.

```
// hud.hpp (C++)
// #include <cstdio>
// namespace ui {
// class Counter {
// public:
//     Counter() noexcept;
//     void show(int value) noexcept { std::printf("int %d\n", value); }
//     void show(long long value) noexcept { std::printf("long long %lld\n", value); }
//     void show(double value) noexcept { std::printf("double %.2f\n", value); }
// };
// }
import cpp "hud.hpp" with (project="game") as hud

fn main():
    lives: u8 = 3
    # SAFETY: the counter only prints the number it is given
    unsafe:
        c = hud.ui.Counter.new()
        c.show(lives)
```

* **not_general**, medium: C++ prints `int 5` on every target. Under the edit the same Ember program compiles on Linux and is refused on Windows, and the C++ overload sets that cover every integer type (`QString::number`, `std::to_string`, `abs`) cannot be used at all on one target or the other. This also re-tests E1-12: its premise, that identity is undecided, is refuted, because the library says aliases. The failure comes from the edit ranking on the aliased Ember types, while its literal rule already appeals to C++'s own types. Under the strict test (refused today too) a checker may class it pre_existing; it is filed here because decision iii states this program's outcome as one its words give.

```
// stream.hpp (C++)
// #include <cstdio>
// namespace io {
// class Out {
// public:
//     Out() noexcept;
//     void put(int v) noexcept { std::printf("int %d\n", v); }
//     void put(long v) noexcept { std::printf("long %ld\n", v); }
// };
// }
import cpp "stream.hpp" with (project="tools") as s

fn main():
    # SAFETY: `put` only prints its argument
    unsafe:
        out = s.io.Out.new()
        out.put(5)
```

New gaps found on the way (not this edit's): 6

Tests rejected by the script, not written (10):
* override_of_overloaded_virtual: the expected output is a description, not the exact text: C++'s meaning: stdout "36 108\n" once, and draw(const Label&) stays the base's. Undecided 
* widget_override_mesh_slot: the expected output is a description, not the exact text: 36 108 (printed once)
* keylog_upcast_override_refused: the program is a description, not code: (hole H1's program, whole: input.hpp with Event, KeyEvent, FocusEvent, Handler with on(con
* scene_per_argument_ranking: the program is a description, not code: (hole H2's program, whole: scene.hpp with set(int, int), set(int, float), const/non-const 
* config_out_of_line_const_pair: the program is a description, not code: (hole H3's program, whole: config.hpp with both `at` declared, bodies in config.cpp; overl
* voice_deleted_overload: the program is a description, not code: (hole H4's program, whole: voice.hpp with set_volume(float linear) and set_volume(int) = d
* server_this_field_choice: the program is a description, not code: (hole H9's program, whole: config.hpp with inline const/non-const at; overlays/config.em; 
* inventory_const_assertion_on_overload: the program is a description, not code: (hole H7's program, whole: legacy.hpp with non-const count() and count(int kind); overlays
* sink_rvalue_virtual_override: the program is a description, not code: (hole H5's program, whole: jobs.hpp with Payload (move ctor, dtor), Sink with virtual take
* keylog_single_virtual_mismatch: the program is a description, not code: (hole H6's program, whole: input.hpp with Handler's single virtual on(const Event& e); cla

To verify on the compiler:
* Renderer: `import cpp` of renderer.hpp (two constructors, two `draw`) gives no `E1030`. `Renderer.new(1280, 720)` runs `Renderer(int, int)`, and `r.draw(mesh)` / `r.draw(hud)` run `draw(const Mesh&)` / `draw(const Sprite&)`. Check with `ember bind --emit-cpp` and a run that prints which overload ran.
* Shader: `s.set(c"light_count", 4)` runs the `int` overload and `s.set(c"exposure", 1.5)` the `float` one. The untyped entry `fn set(name: borrowed nul_terminated, value)` is `E5010`, naming both. `super.init(16, 8)` under `Widget(int, int)` / `Widget(float, float)` runs `(int, int)`.
* Literals: `out.put(5)` under `put(int)` / `put(long)` runs `put(int)`, and `m.scale(2.0)` under `scale(float)` / `scale(double)` runs `scale(double)`. `w.value(7)` under `value(int64_t)` / `value(double)` is `E2070`, naming both. Compare each case with MSVC, Clang and GCC on Windows and Linux, since `int64_t` is `long long` on one and `long` on the other.
* Asset cache: `cache.add(atlas)` runs `add(const Texture&)` and leaves `atlas` usable; `cache.add(owns splash)` runs `add(Texture&&)`. A second `cache.add(owns splash)` is `E3040` at that line, not `E2070`.
* Config: `cfg.at(0).port = 8080` through a changeable local runs the non-const `at`; through a read-only parameter, the const one runs; the program prints 8080. Two non-const results held at once through one changeable name are refused (E3022): check the message.
* Mixer: `ring.push(pcm)` (an `i16`) runs `push(int16_t)` both inside and outside `unsafe:`. Outside, it is `E3100` naming that function. The lead's wording should also name each safe overload the call would reach with another argument type (`m.gain(1.5 as f32)`).
* Effects: in a `@noalloc` function, the same call picks the same overload as elsewhere, and `E4001` is given on that overload if it lacks `effects=[]`.
* Override: `override fn on(e: gui.ui.KeyEvent)` replaces only `on(const KeyEvent&)` in the trampoline, while the click runs the base's `on(const MouseEvent&)`; the program prints 1. With the widget header, `36 108` is printed once. Check the generated C++ with `ember bind --emit-cpp`. Also check whether Ember code still reaches the base's other overload through the derived class.
* Function value: `project: fn(vm.Vec3i, vm.Vec3i) -> c_int = vm.dot` prints 32. `f = vm.dot` with no expected type is `E2070`, naming both.
* Redeclaration: a member declared in its class and defined below it, or a function declared twice, is one function (one USR). No `E2070` and no `E5010`.
* Overlay record: the renderer overlay gives no `E5010`. `ember.overlays` keys each entry by its function (the key format is the lead's tooling). A new overload whose parameter has the same name makes the old entry `E5010`, naming both.
* Const pair in an overlay: `@ffi(const) fn visit(ref mut v: Visitor)` describes the const `visit` and `fn visit(ref mut v: Visitor)` the other; `root.visit(ref mut tally)` through a read-only `root` runs the const one.
* Typed entries: how a type named like a contract word (`handle`, `one`, `status`) is read before the contract words. The lead decides the reading, for example a raw identifier `r#handle`.
* `L3015` for an overloaded C++ name comes once per function and names every other call. Its message gives an entry line that compiles, with types where the names do not tell the functions apart.
* `E2070`'s message for an imported overload set (the code table says 'ambiguous interface method') is the lead's wording. It names each C++ declaration and why each does not fit; where all fail for one reason (E3023 for every overload), it gives that reason.
* Whether `c_int` and `c_long` are distinct Ember types on MSVC (pre-existing, E1-12). If they are aliases, `put(int)` / `put(long)` import with one signature and every call ties.

