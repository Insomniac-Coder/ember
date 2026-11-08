# New gaps the reviewers found on the way (not de-duplicated by the gap review)

## NG-01 (from deep/v3/results/pilot.json)
kind: undecided
caused_by: pre_existing
severity: high
Program:
```
from std.mem import Allocator, Layout, System

struct CountingPool:
    out: int

# SAFETY: every block comes from System and goes back to it unchanged
unsafe extend CountingPool implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.out += 1
        return System().allocate(layout)

    unsafe fn deallocate(p: *mut u8, layout: Layout):
        this.out -= 1
        System().deallocate(p, layout)

    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return System().reallocate(p, old, new_size)

class Bullet:
    x: int

class Gun:
    last: weakref Bullet = None

struct Armory:
    pool: Option[CountingPool] = None

fn fire(ref mut gun: Gun) -> CountingPool:
    pool = CountingPool(out=0)
    b = Bullet(x=7) using pool          # the bullet keeps one word: the address of this frame's `pool`
    gun.last = weakref b                # the link keeps the bullet's block
    println(b.x)
    mem.drop(owns b)                    # the bullet ends; its block, with the word, is kept by the link
    return pool                         # the instance moves to the caller; the word still names this frame

fn main():
    armory = Armory()
    gun = Gun()
    armory.pool = Some(fire(ref mut gun))   # the instance lives in `armory`, which outlives `gun`
    if gun.last exists as b:
        println(b.x)
    else:
        println("gone")
    # gun ends: the weak count reaches 0 and the block goes back through the word, into fire's ended frame
```
today: It compiles. Nothing bounds the link. While the bullet lived, its word was a kept sharedref mut link to `pool`
(p07 620-623 [LT-4]), but nothing keeps that hold once the bullet has ended. At gun's end, deallocate runs
through a word that points into `fire`'s ended frame, so `this.out -= 1` writes a dead stack slot (UB), and
the real instance's count stays 1. In C++ terms: a `polymorphic_allocator` that kept the address of a
`memory_resource` which was then moved out of its stack slot.
with_edit: Undecided. Read with the instance following the move, the edited LT-4 is satisfied: the instance now lives in
`armory`, which outlives `gun`. The edited ALC-5 does not say that moving the instance is a 'change'. The
program is then accepted and has the same UB. Read with the instance as fire's local, it is E3063 at `gun.last
= weakref b`. The edit's verdict-47 note says the last link's give-back 'always reaches a live instance', but
live is not the same as at the same address.
what_goes_wrong: A block kept after its value ends still needs its instance at the address its word (or, for a FoundByAddress
manager, its piece header) names. The edit bounds how long the instance lives, not where it is. So for every
manager that keeps a word, and for FoundByAddress, moving the instance between the value's end and the last
link's end sends the give-back to dead memory.
rule_lines: p07-ownership.md 620-623 [LT-4] ('that address is a kept `sharedref mut` link to the instance: the program may
use the instance while values made in it live'); p09-memory.md 242-245 [ALC-5] as edited; p09-memory.md
204-211 [ALC-1] (which values keep the word); p09-memory.md 231-235 [ALC-2] (FoundByAddress keeps the
instance's address in each piece)
fix_hint: A block kept after its value ends keeps that value's hold on its instance: the instance is neither moved nor
ended while the block is kept.

## NG-02 (from deep/v3/results/pilot.json)
kind: undecided
caused_by: pre_existing
severity: medium
Program:
```
struct Mesh:
    points: Array[float]

class Enemy:
    hp: int
    mesh: owns Mesh using Arena

class Camera:
    target: weakref Enemy = None

fn load_level(ref mut cam: Camera):
    level = Arena.with_capacity(64 * 1024)
    boss = Enemy(hp=500, mesh=owns Mesh(points=[0.0, 1.0]) using level)   # the boss from the built-in manager, its mesh from `level`
    cam.target = weakref boss
    println(boss.hp)

fn main():
    cam = Camera()
    load_level(ref mut cam)
    if cam.target exists as e:
        println(e.hp)
    else:
        println("gone")
```
today: It compiles and is correct: boss ends first, its mesh goes back to `level` while `level` lives, and the boss's
block, from the built-in manager, is kept by the link. It prints 500, then gone. That holds only while LT-4's
'every value that holds one' is not read as covering a link. The edit's own notes argue that a link 'holds'
its target (WK-11 p08 328). Read transitively, the link holds the boss, which holds a mesh made in `level`, so
the link would be bounded by `level`.
with_edit: The added clause 'or keeps its block' decides the link through the block it keeps, here the built-in
manager's, so the program compiles. The older 'holds one' clause is unchanged, though, and under the notes'
reading it still refuses the program (E3063 at `cam.target = weakref boss`).
what_goes_wrong: Under the reading the edit's own rationale gives, an ordinary game program is refused although it is correct:
verdict 24's own layout (boss from the built-in manager, mesh from the arena) with a camera link. That would
be a needless refusal on the language's most common arena layout.
rule_lines: p07-ownership.md 613-615 [LT-4] ('and every value that holds one'); p08-classes.md 326-331 [WK-11] ('A link
may hold no target'; 'holds one weak count'); JOB.md 216-221 (verdict 24)
fix_hint: Say that a link is bounded only by the instance of the block it keeps, never by what its target holds.

## NG-03 (from deep/v3/results/pilot.json)
kind: undecided
caused_by: pre_existing
severity: high
Program:
```
class Enemy:
    hp: int

fn main():
    level = Arena.with_capacity(64 * 1024)
    links: Array[weakref Enemy] = []
    a = Enemy(hp=1)                      # from the built-in manager
    b = Enemy(hp=2) using level          # from `level`
    links.push(weakref a)
    links.push(weakref b)
    mem.drop(owns a)
    mem.drop(owns b)
    links.clear()                        # each last link ends: each block goes back to its own manager
    println(links.len())
```
today: Compiles. At `links.clear()` the last weak count of each target reaches 0 and 'the block is freed' (OBJ-3 p08
85). But a `weakref Enemy` place names no manager (grammar p03 167-170: `using` binds the whole `weakref
Enemy`, which is E2114 by ALC-1), and the header carries 'no flags but a `thread_shared` value's three marks'
(OBJ-1 p08 72-73). So nothing written says whether `a`'s block goes to the built-in manager and `b`'s to
`level`, or how the code at one list element knows which. It is the same for a manager whose values keep a
word: the code cannot tell the word is there.
with_edit: The same. The edit bounds both links by `level`, which `links` satisfies (made after it), but the give-back at
the end of each link is still not decided.
what_goes_wrong: Whichever give-back an implementation picks for a mixed list is wrong for one element: a built-in block handed
to the arena (a leak), or an arena block handed to `free` (heap corruption, like `free()` on a pointer from a
`std::pmr` arena). The edit's verdict-47 'how' ('the last link's give-back of the block can rely on that word
(or on kind 1 or 2)') assumes the kind is known at the link's end.
rule_lines: p03-grammar.md 167-170 (type := base_type ["using" path_type]; "weakref" base_type); p09-memory.md 199-201
[ALC-1] ('`using` after any other type is `E2114`'); p08-classes.md 63-73 [OBJ-1]; p08-classes.md 83-86
[OBJ-3]; p08-classes.md 87-90 [OBJ-4]
fix_hint: The block's give-back is decided by where the block came from, found from the block itself (as FoundByAddress
or the chunk header does), never by the link's static type. This is the finder's gap (2), for the list.

## NG-04 (from deep/v3/results/pilot.json)
kind: unsafe_allowed
caused_by: pre_existing
severity: medium
Program:
```
struct Counting:
    live: int

# SAFETY: every block comes from the platform's allocator, valid and unaliased until it is given back
unsafe extend Counting implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.live += 1
        return System().allocate(layout)

    @safety("`p` came from `allocate` with this `layout`")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        this.live -= 1
        System().deallocate(p, layout)

    @safety("`p` came from `allocate` with the layout `old`")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return System().reallocate(p, old, new_size)

class Enemy:
    hp: int

class Camera:
    target: weakref Enemy = None

fn main():
    cam = Camera()
    counter = Counting(live=0)
    boss = Enemy(hp=500) using counter   # the boss's block keeps `counter`'s address
    cam.target = weakref boss
    mem.drop(owns boss)                  # the boss ends; the link keeps its block, and the address in it
    moved = counter                      # the instance is handed over to another place
    println(moved.live)
    cam.target = None                    # the last link ends: the block goes back through the old address
```
today: Compiles. LT-4 p07 620-623 makes the instance address 'a kept `sharedref mut` link to the instance ... while
values made in it live'. Once `boss` ends, nothing keeps the instance in place, so `moved = counter` is
allowed. The give-back at `cam.target = None` then calls `deallocate` through the address of the moved-from
`counter`.
with_edit: The same. The link ends while the instance lives on under `moved`, so the edited LT-4 is satisfied. A
hand-over is neither 'a change' in the edited ALC-5's sense nor an end of the instance. The address in the
kept block is still stale.
what_goes_wrong: `deallocate` runs on a dead place: a read and write of the moved-from `live`, like a `std::pmr` resource moved
while a block still points back at it. `moved.live` stays 1 for ever. The edit's verdict-47 'how' ('always
reaches a live instance') does not hold for a moved instance.
rule_lines: p07-ownership.md 620-623 [LT-4] ('that address is a kept `sharedref mut` link to the instance (`[LT-46]`): the
program may use the instance while values made in it live'); p09-memory.md 242-243 [ALC-5] (edited);
p08-classes.md 83-86 [OBJ-3]
fix_hint: A block's instance address stays a kept link to the instance for as long as the block is kept, not only while
its value lives; then the hand-over is refused like the one under a live value.

## NG-05 (from deep/v3/results/pilot.json)
kind: undecided
caused_by: pre_existing
severity: medium
Program:
```
# ember.toml:
# [build]
# global_allocator = "tally.TALLY"

# src/tally.em
from std.mem import Allocator, Layout, System
from std.sync import Atomic

pub struct Tally:
    blocks: Atomic[int]

# SAFETY: every block comes from the platform's allocator, which any thread may use
unsafe extend Tally implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        this.blocks.fetch_add(1)
        return System().allocate(layout)

    @safety("`p` came from `allocate` with `layout` and is not used again")
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        System().deallocate(p, layout)

    @safety("`p` came from `allocate` with `old`")
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return System().reallocate(p, old, new_size)

pub static TALLY = Tally(blocks=Atomic(0))

# src/main.em
from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    stats = thread_shared Stats()
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))
    println("done")
```
today: `Global` is the package's configured allocator, here TALLY (ALC-2 p09 224-227; ALC-3 p09 237-239). RT-1 (p18
243-246) says allocation is the system malloc per block, and a host may supply its own allocator (FFI-27 p16
685-688). OBJ-1 (p08 69-71) reads the making thread from 'the header of the memory chunk its block came from'.
No rule says a block TALLY hands out lies in such a chunk, so how the counts and the end run is unspecified.
In C++ terms: a biased refcount reading a garbage owner id.
with_edit: The same. The edit's added words 'only the built-in manager's chunks carry that header' now assert something
no rule makes true for a configured or host allocator.
what_goes_wrong: If the runtime takes each thread_shared block straight from `TALLY.allocate`, there is no record of the making
thread. Two threads may both count as the making thread, which means a race on the home count and a double
end. This happens with or without the edit; the edit's soundness argument rests on the missing rule.
rule_lines: OBJ-1 p08 69-71; ALC-2 p09 224-227; ALC-3 p09 237-239; RT-1 p18 243-246; FFI-27 p16 685-688
fix_hint: State that the runtime keeps every thread_shared block in chunks it makes and records itself, whatever
allocator the package or host supplies.

## NG-06 (from deep/v3/results/pilot.json)
kind: undecided
caused_by: pre_existing
severity: medium
Program:
```
from std.sync import Atomic
import std.thread

class Stats:
    hits: Atomic[int] = Atomic(0)

fn load_stats(ref mut current: Arena) -> Stats using Arena:
    return Stats() using current

fn main():
    level = Arena.with_capacity(64 * 1024)
    stats = thread_shared load_stats(ref mut level)
    with scope = thread.scope():
        scope.spawn(owns fn() owns stats: stats.hits.fetch_add(1))
    println("done")
```
today: Two readings. OWN-13 (p07 139-140) allows `thread_shared` 'before a new value', and BRW-12 (p07 200-201) calls
a call's result 'a new value the callee made'. Under that reading, HEAP-3 (p09 60-61, 'A class object already
has its block') keeps the arena block, and OBJ-1 (p08 69-71) reads the maker from a chunk header that block
lacks: undefined, exactly as in J5-05. But THR-1 (p11 71-72) says 'on a value made otherwise it is E7017', and
E7017's row (p17a 219) speaks of 'a name'. Under that reading line 12 is E7017.
with_edit: The same two readings. No `using` follows `thread_shared` on line 12, and line 7's `Stats using Arena` is a
class type, which the shortened list still allows, so the new E2114 never fires. Under the first reading, the
new sentence ("A `thread_shared` value's block always comes from the built-in manager") contradicts HEAP-3.
The only fix the edit offers ('remove `using`') has no word to remove here.
what_goes_wrong: The spec does not say whether line 12 is refused, keeps the arena block (undefined, the J5-05 danger), or
moves the object into a new built-in block. Only the E7017 reading makes the edit's 'every place' claim hold
for class objects.
rule_lines: p07-ownership.md 135-140 (OWN-13), 200-201 (BRW-12); p11-concurrency.md 69-74 (THR-1); p17a-codes.md 219;
p09-memory.md 58-62 (HEAP-3); p08-classes.md 69-71 (OBJ-1)
fix_hint: Decide where the header is chosen: `thread_shared` before a call's result is E7017 unless the result's type is
already thread_shared, which the edit then covers through the result type.

## NG-07 (from deep/v3/results/pilot.json)
kind: undecided
caused_by: pre_existing
severity: medium
Program:
```
# ember.toml
[package]
name = "app"
[build]
global_allocator = "app.TRACED"

# app.ember
from std.mem import Allocator, Layout
from std.sync import Atomic
import std.thread

struct Traced:
    pass

# SAFETY: every block comes from System, which gives valid, aligned, unaliased memory until it is given back
unsafe extend Traced implements Allocator:
    fn allocate(layout: Layout) -> *mut u8:
        return System().allocate(layout)
    unsafe fn deallocate(p: *mut u8, layout: Layout):
        System().deallocate(p, layout)
    unsafe fn reallocate(p: *mut u8, old: Layout, new_size: usize) -> *mut u8:
        return System().reallocate(p, old, new_size)

static TRACED: Traced = Traced()

class Stats:
    hits: Atomic[int] = Atomic(0)

fn main():
    stats = thread_shared Stats()
    with scope = thread.scope():
        scope.spawn(owns fn() thread_shared stats: stats.hits.fetch_add(1))
    println(stats.hits.load())
```
today: OBJ-1 (p08 69-71) reads the making thread 'from the header of the memory chunk its block came from, written
once when the chunk is made', but no rule says the built-in path makes chunks. RT-1 (p18 243-246) says
allocation is the system malloc, or a host's functions (FFI-27, p16 685-686). ALC-2 (p09 224-225) makes
`Global` the allocator that ALC-3 names (p09 237-238), here the program's own `Traced`, whose blocks are
System's. Where `stats`'s making thread is read from is undefined.
with_edit: The same. The new sentence ('only the built-in manager's chunks carry that header') asserts it, but no rule
makes it so.
what_goes_wrong: In a package with its own global allocator, or under a host's allocation functions, every thread_shared value
reads its maker from a header that no rule says is written. The edit's reason rests on this unstated runtime
layer. Its to_verify asks the lead to check it on the runtime, but the specification text has the gap either
way.
rule_lines: p08-classes.md 69-71 (OBJ-1); p18-implementation.md 243-246 (RT-1); p09-memory.md 224-225, 237-239 (ALC-2,
ALC-3); p16-ffi.md 685-686 (FFI-27)
fix_hint: State in the runtime rules that every block for a value made with no `using` comes from a runtime chunk carved
from the global allocator, whose header records the making thread.

## NG-08 (from deep/v3/results/pilot.json)
kind: programmer_must_write_what_compiler_could_infer
caused_by: pre_existing
severity: low
Program:
```
import std.thread

struct Parser:
    keywords: thread_shared Array[String] using System

fn main():
    p = Parser(keywords=thread_shared [String("let"), String("fn")] using System)
    with scope = thread.scope():
        scope.spawn(owns fn() ref p: println(p.keywords.len()))
```
today: GRM-3 (p03 189-190) makes line 4's `using System` name the block of the whole `thread_shared Array[String]`,
its shared block, and the buffer comes from Global. GRM-43 (p03 435-438) and p06 108-109 make line 7's `using
System` name the list's buffer. The field and the value differ, so it is E2020, or, if line 4 is read as the
buffer, it compiles. The type of a shared list whose buffer is in a manager has no direct spelling, because
`base_type` after `thread_shared` takes no `using` (p03 169). The same holds for `owns [..] using current`
against `owns Array[T] using A`.
with_edit: Line 4 is E2114 (header block). Removing `using System` makes line 7 E2020 (a System buffer against a Global
one). The only spelling left is a transparent alias, `type SysWords = Array[String] using System` with
`keywords: thread_shared SysWords`, which the edit's property allows.
what_goes_wrong: The making line accepts a value whose type cannot be written plainly in a field, parameter or result. A
program has to declare an alias to store it. The edit's code changes (a mismatch becomes E2114) but the gap
does not, so it is not a hole of this edit and goes on the list.
rule_lines: GRM-3 p03 189-190; p03 167-169; GRM-43 p03 435-438; p06 108-109; TYP-46 p04 440-446
fix_hint: Give the type grammar the same binding as the making line, a `using` that names the inner value's own memory
under a word (owns, shares, thread_shared), so the field spelling exists without an alias.

## NG-09 (from deep/v3/results/pilot.json)
kind: unsafe_allowed
caused_by: pre_existing
severity: medium
Program:
```
import std.thread

static mut SAMPLES: Array[f32] = []            # the mix buffer: a plain std::vector<float>, main's own

class Voice:                                   # one playing voice
    id: int
    fn drop():                                 # a finished voice leaves one silent sample
        SAMPLES.push(owns 0.0)

class Chord:                                   # a plain object; its voices travel inside it
    voices: Array[thread_shared Voice] = []

fn settle(c: Chord) -> int:                    # a scoped task's work; the chord ends with the task
    total = 0
    for i in 0..1000000:
        total += i % 3
    return total + c.voices.len()

fn main():
    for i in 0..1024:
        SAMPLES.push(owns 1.0)
    c = Chord(voices=[thread_shared Voice(id=1)])
    with s = thread.scope():
        s.spawn(owns fn() owns c: settle(c))   # the task ends c when it finishes: Voice 1 goes back to main
    p = SAMPLES.as_mut_ptr()
    n = SAMPLES.len()
    for i in 0..n:                             # a back edge: Voice's drop may grow SAMPLES here
        # SAFETY: p points at the n samples of SAMPLES, and nothing in this loop changes SAMPLES
        unsafe:
            p.add(i).write(0.5)                # after the drop grew SAMPLES, this writes into freed memory
    println(SAMPLES.len())
```
today: Compiles: a counting loop may add to SAMPLES (BRW-18 p07 527-531), extracting the pointer is safe (SPN-8 p07
837), and a scoped task may reach main's static (STA-1 p05 478-480). The Voice is queued to main while main
waits at the scope's end. There is no wake today, so main's next check point is the unsafe loop's first back
edge (RC-4 point 3, p08 169-171). Voice's drop grows SAMPLES there, and every later write goes through p into
the freed buffer.
with_edit: The same. A push to SAMPLES in a counting loop is not refused, so the check point stays inside the unsafe
loop. The wake saves this timing only under the narrow reading, where the check point before the scope's end
is placed. Under the broad reading it is left out (THR-11), and the program writes freed memory as today.
what_goes_wrong: A use of freed memory in unsafe code whose own reasoning was right: the author could not see the compiler's
hidden call at the back edge. Rust puts no hidden call in a loop. Not this edit's fault; a new gap for the
list.
rule_lines: RC-4 point 3 p08 169-171 (today and as edited); SPN-8 p07 837; UNS rules p09 263-275; BRW-18 p07 527-531.
fix_hint: No hidden call is placed inside unsafe code: wherever the programmer vouches for memory by hand, a check point
counts as refused.

## NG-10 (from deep/v3/results/pilot.json)
kind: not_general
caused_by: pre_existing
severity: medium
Program:
```
import std.thread
from std.ffi import ForeignCallback, Retained

class Texture:                                # a GPU texture made on the UI thread
    id: int
    fn drop():
        println("texture", this.id, "freed")  # in an engine: hand the GPU memory back

class Upload:                                 # a plain object, handed to the upload thread whole
    textures: Array[thread_shared Texture] = []

unsafe extern "C":
    @safety("`cb` and `ud` are a pair `on_frame` was given")
    unsafe fn remove_frame_cb(cb: extern "C" fn(*mut void), ud: *mut void)
    @ffi(param(cb, callback(retained, user_data=ud), threads(main)), destroys(remove_frame_cb))
    safe fn on_frame(cb: extern "C" fn(*mut void), ud: *mut void)
    safe fn ui_run()                          # the toolkit's main loop (as gtk_main): calls the frame callback, returns when the window closes

fn draw():
    println("frame")

fn main():
    u = Upload(textures=[thread_shared Texture(id=1)])
    h = thread.spawn(owns fn() owns u: u.textures.len())   # the upload thread ends u: Texture 1 goes back to main
    token = on_frame(fn(): draw())
    ui_run()                                  # main now lives in C's loop; C calls draw every frame
    println(h.join())
```
today: Main passes its check point before `ui_run()` and then lives inside C. `draw` has no loop and no call that may
block, and each callback returns into C with main's own Ember frame still below it. So no check point is
reached, and Texture 1 waits until the window closes (the check point before `h.join()`).
with_edit: The same. The runtime's new point is 'returns to C from its outermost Ember call', but `draw` is not the
outermost Ember call (main is). Main is inside C, not in a wait the runtime performs, so it is not woken.
Texture 1's GPU memory stays held for as long as the window is open.
what_goes_wrong: J7-02's defect stays for the most common C-driven architecture: a UI thread whose loop lives in C (GTK, Cocoa,
Android's looper, libuv). The edit's own count already makes a check point at every return to C safe, because
places still open in the Ember frames below keep the count raised. So the restriction to 'outermost' buys
nothing. This is not caused by this edit (the same program behaves the same today), but the edit's property
misses it at no extra cost.
rule_lines: RC-4 point 3 as edited (p08 169-171): 'the runtime where a thread ... returns to C from its outermost Ember
call'. FFI-21 p16-ffi.md 400-423. FFI-52 p16 451-477. The question's own text: 'an object made in a callback
on a thread C owns, once that thread has gone back into C'.
fix_hint: Every return from Ember into C is a check point; the thread's count keeps it from ending anything inside
places still open in the Ember frames below.

## NG-11 (from deep/v3/results/test2.json)
kind: not_general
caused_by: pre_existing
severity: high
Program:
```
// netq.c, compiled into the program:
//   #include <stdint.h>
//   #include <stddef.h>
//   #include <string.h>
//   void netq_drain(uint8_t* (*next_buffer)(size_t want), size_t want) {
//       uint8_t* p = next_buffer(want);
//       memset(p, 0xAB, want);      /* copies the next packet into the buffer it was handed */
//   }

unsafe extern "C":  # SAFETY(ffi): the declaration below matches netq.c
    @ffi(param(next_buffer, callback(borrowed)))
    safe fn netq_drain(next_buffer: extern "C" fn(usize) -> *mut u8, want: usize)

fn scratch(want: usize) -> *mut u8:
    one: u8 = 0
    return ref_to_ptr(ref mut one)       # safe: a raw pointer keeps no view

fn main():
    netq_drain(scratch, 1500)            # C writes 1,500 bytes into `scratch`'s finished stack frame
    println("drained")
```
today: Compiles. `next_buffer` is a C function-pointer parameter with a complete callback contract (p16-ffi.md 59-61,
400-402: a capture-free Ember function is accepted), and lines 82-84 refuse only a raw-pointer parameter or
result. Returning `ref_to_ptr` of a local is safe (p04-types.md 340-345). FFI-21a (p16-ffi.md 424-434) refuses
only a pointer into a parameter's copy. C writes through a dangling stack pointer: a use after return, from
safe code.
with_edit: Unchanged. A callable type's result is not 'a field, element or payload' of `next_buffer`'s Ember type, so the
edited words do not reach it. It compiles, with the same corruption.
what_goes_wrong: A raw pointer chosen by safe code reaches C as a callback's result, and C writes through it. The edit's own
reason holds for this path ('safe code may point one at any value'), but its words follow only values passed
into C, not values Ember hands back to C.
rule_lines: p16-ffi.md 82-84 (FFI-10, as edited), 400-423 (FFI-21), 424-434 (FFI-21a); p04-types.md 340-345
fix_hint: Judge by direction: every raw pointer that safe Ember code hands to C (an argument, a field, a pointee, or
what a callback returns or writes back) is refused; one that C hands to Ember is not.

## NG-12 (from deep/v3/results/test2.json)
kind: not_general
caused_by: pre_existing
severity: high
Program:
```
// jobs.h
#include <stdint.h>
struct job { void (*run)(uintptr_t arg); uintptr_t arg; };
void jobs_run_now(const struct job* j);   /* calls j->run(j->arg) */
void pool_release(uintptr_t addr);        /* frees the pool block at addr */

# overlays/jobs.em
unsafe overlay c "jobs.h":
    fn jobs_run_now(j: borrowed one)
    @safety("`addr` is a block `pool_take` returned, not released yet")
    unsafe fn pool_release(addr)

# main.em
import c "jobs.h" with (overlay = "overlays/jobs.em") as jobs

fn main():
    jobs.jobs_run_now(jobs.job(run=jobs.pool_release, arg=4096))   # C releases address 4096
    println("done")
```
today: Compiles. The overlay entry asserts `jobs_run_now` safe (p16-ffi.md 33-40), and `job` holds no raw pointer.
`job.run` imports as `unsafe extern "C" fn(usize)` (p16-ffi.md 590-592). An `unsafe fn` coerces to that unsafe
callable type (p05-declarations.md 182-183), and only calls are unsafe, so `run=jobs.pool_release` is safe
code. C calls `pool_release(4096)`: an invalid free. `pool_release`'s `@safety` was never vouched.
with_edit: Unchanged. The edited FFI-10 words and FFI-56's new list reach raw pointers only, and `job` holds none.
what_goes_wrong: Safe code chooses which unsafe C function C will call, so that function's precondition is never vouched. It is
the same property as a raw pointer: nothing true can be asserted about an arbitrary value. The edit's note
leaves `extern "C" fn` fields out on the ground that FFI-52 counts every Ember function given to C, but that
reason does not cover an `unsafe extern "C" fn` field, which holds a C function.
rule_lines: p16-ffi.md 82-84 (FFI-10, as edited), 590-598 (FFI-56, as edited); p05-declarations.md 182-183 (FN-6)
fix_hint: An `unsafe extern "C" fn` held at any depth in what a safe declaration exposes is judged like a raw pointer; a
safe `extern "C" fn` is not.

## NG-13 (from deep/v3/results/test2.json)
kind: undecided
caused_by: pre_existing
severity: high
Program:
```
// mixer.hpp
#include <cstdint>
#include <functional>
struct Block { float* samples; uint32_t count; };
using BlockFn = std::function<void(Block)>;
BlockFn make_gain(float g) noexcept;     // the result scales `count` samples by g

# overlays/mixer.em
unsafe overlay cpp "mixer.hpp":
    @ffi(std_function)
    type BlockFn
    @ffi(no_callbacks)
    fn make_gain(g)

# main.em
import cpp "mixer.hpp" with (overlay = "overlays/mixer.em") as mx

fn main():
    one: f32 = 1.0
    gain = mx.make_gain(0.5)
    gain(mx.Block(samples=ref_to_ptr(ref mut one), count=4096))   # C++ scales 4,096 floats starting at `one`
    println(one)
```
today: Compiles. With `@ffi(std_function)`, `BlockFn` is a function value `fn(Block)`, 'called as any other'
(p21-annex-c.md 67). Nothing checks its parameter types: here a struct with a raw pointer, and equally a
direct `float*`. C++ writes 4,096 floats over one: a buffer overflow from safe code.
with_edit: Undecided. The edited sentence refuses 'a declaration asserting a foreign function safe to call (`[FFI-2]`,
`[FFI-56]`)'. An `@ffi(std_function)` type entry asserts exactly that, but neither cited rule names it, and
FFI-56's edited list covers C typedefs only.
what_goes_wrong: A third kind of declaration makes foreign code callable from safe code, and it escapes the raw-pointer
property, struct or not. The edit's claim that its words reach 'every kind of declaration that asserts safety'
does not hold for it.
rule_lines: p16-ffi.md 82-84 (as edited), 590-598 (as edited); p21-annex-c.md 67
fix_hint: Every declaration that makes foreign code callable from safe code (`safe fn`, an overlay entry, `@ffi(safe)`,
`@ffi(std_function)`) is judged by the one raw-pointer property.

## NG-14 (from deep/v3/results/test2.json)
kind: not_general
caused_by: pre_existing
severity: high
Program:
```
// blob.h
#include <stddef.h>
#include <stdint.h>
struct blob { uint8_t* data; size_t len; };
struct blob* blob_new(size_t len);    /* malloc'd blob whose data holds len malloc'd bytes */
void blob_free(struct blob* b);       /* free(b->data); free(b); */

# overlays/blob.em
unsafe overlay c "blob.h":
    @ffi(destroys(blob_free))
    type blob
    @safety("returns a new blob whose `data` holds `len` bytes, or null when out of memory")
    unsafe fn blob_new(len)
    @safety("`b` is a blob `blob_new` made, not freed yet")
    unsafe fn blob_free(b)

# main.em
from std.ffi import adopt
import c "blob.h" with (overlay = "overlays/blob.em") as bl

fn make(len: usize) -> owns bl.blob:
    # SAFETY: `blob_new` gives a fresh blob, checked non-null, that only this owner ends
    unsafe:
        p = bl.blob_new(len)
        assert(p != null[*mut bl.blob]())
        return adopt[bl.blob](p)

fn main():
    first = make(64)
    second = make(64)
    second.data = first.data      # safe code: copying a raw pointer value
    println(first.len + second.len)
    # the ends call blob_free(second), then blob_free(first): first's bytes are freed twice
```
today: Compiles and prints 128, then frees the same block twice (and leaks second's bytes). `adopt` is unsafe and
vouched for the blob as it was made (p16-ffi.md 386-393). Afterwards, assigning a raw-pointer field is safe
code (p04-types.md 340-345). The end of each `owns bl.blob` calls the destructor that `destroys(blob_free)`
asserts (p16-ffi.md 599-601).
with_edit: Unchanged. The edit refuses the safe `returns_owned` form of `blob_new` (its result's pointee holds a raw
pointer), but not the `destroys(f)` end. A `destroys(f)` end is not a declaration that FFI-2 or FFI-56 names.
what_goes_wrong: An asserted foreign end acts on a raw pointer that safe code rewrote: a double free (C++: double delete). It
is the edit's property (C acts on an address safe code chose), reached by a route the edited words do not
follow.
rule_lines: p16-ffi.md 82-84 (as edited), 386-393 (FFI-36a), 599-601 (FFI-56's last sentence); p04-types.md 340-345
fix_hint: An asserted end (`destroys(f)`) of a type whose fields hold a raw pointer that safe code may rewrite is the
same assertion, and is judged alike.

## NG-15 (from deep/v3/results/test2.json)
kind: undecided
caused_by: pre_existing
severity: medium
Program:
```
// tex.h
typedef struct tex* tex_ref;            /* a texture handle */
struct draw_cmd { tex_ref tex; float x; float y; };
void tex_draw(struct draw_cmd cmd);     /* reads cmd.tex->width and cmd.tex->pixels */

# overlays/tex.em
unsafe overlay c "tex.h":
    @ffi(newtype)
    type tex_ref
    @ffi(no_callbacks)
    fn tex_draw(cmd)

# main.em
import c "tex.h" with (overlay = "overlays/tex.em") as tx

fn main():
    bogus = null[*mut tx.tex]() as tx.tex_ref        # a nominal type, converted with `as`
    tx.tex_draw(tx.draw_cmd(tex=bogus, x=0.0, y=0.0))  # C reads through a null texture
    println("drawn")
```
today: Not decided. `tex_ref` is 'a nominal type with the typedef's representation, ... converted with `as` only'
(p16-ffi.md 185). Nothing says whether a newtype over a C pointer counts as a raw pointer for lines 82-84, or
whether the `as` from `*mut tex` is safe (p04-types.md 345-347 names raw-to-raw and pointer/integer casts
only). If accepted, C dereferences a null pointer, or a stale copy of a freed handle, from safe code.
with_edit: Still undecided. 'held anywhere in one's Ember type (a field, element or payload ...)' does not say whether a
newtype's representation is held. The edit's notes leave handles to K46, but its words do not.
what_goes_wrong: A pointer under another name may slip past the raw-pointer property in a struct field or as a direct
parameter. Whether it is refused depends on a reading the rule does not settle.
rule_lines: p16-ffi.md 82-84 (as edited), 185 (FFI-8 typedef row); p04-types.md 345-347
fix_hint: A type whose values safe code can turn into any address counts as a raw pointer, whatever its name, unless a
contract says what C may do with it.

## NG-16 (from deep/v3/results/test2.json)
kind: unsafe_allowed
caused_by: pre_existing
severity: high
Program:
```
from std.ffi import c_int

@layout(c)
struct PoolConfig:
    block_size: usize
    reap: unsafe extern "C" fn(usize)    # the pool calls it with the address of each block it carved from its own arena

unsafe extern "C":
    @safety("`addr` came from `alloc_addr` and is not freed yet")
    @ffi(no_callbacks)
    unsafe fn free_addr(addr: usize)
    safe fn pool_run(cfg: PoolConfig) -> c_int

fn main():
    println(pool_run(PoolConfig(block_size=64, reap=free_addr)))
```
today: Compiles. PoolConfig is a `@layout(c)` value type (p16 59-61). The value of `free_addr`, an `unsafe fn`,
coerces to an `unsafe` callable type in safe code, and only calling it is unsafe (p05-declarations.md
182-185). C then calls `free_addr` on arena addresses that `alloc_addr` never gave, which corrupts the heap
(in C++ terms, `free` on a pool block).
with_edit: The same. The field holds no raw pointer, so the edited words do not reach it.
what_goes_wrong: Safe code chooses which function C calls, with arguments nothing checks. This is the same property as J8-08
(nothing says where it points), but the edit's words list only raw pointers.
rule_lines: p16-ffi.md 59-61, 83 (as edited), 590-598 (FFI-56); p05-declarations.md 182-185
fix_hint: Count an `unsafe` function value that safe code chose as the edit counts a raw pointer: C calls it with
arguments nothing checks.

## NG-17 (from deep/v3/results/test2.json)
kind: unsafe_allowed
caused_by: pre_existing
severity: high
Program:
```
from std.ffi import c_int

unsafe extern "C":
    # int read_packet(uint8_t* (*make_buf)(size_t want), size_t want): C writes `want` bytes where make_buf points
    @ffi(param(make_buf, callback(borrowed)))
    safe fn read_packet(make_buf: extern "C" fn(usize) -> *mut u8, want: usize) -> c_int

fn scratch(want: usize) -> *mut u8:
    byte: u8 = 0
    return ref_to_ptr(ref mut byte)

fn main():
    println(read_packet(scratch, 1500))
```
today: Compiles. `scratch` is capture-free and coerces to `extern "C" fn` (p05 185-186). FFI-21a (p16 424-434)
refuses only a pointer into a by-value parameter's copy (E5042), not one into a local. C writes 1,500 bytes
into the frame of `scratch` after it has returned (a write through a dangling stack pointer).
with_edit: The same on the parenthetical reading (a function type's result is not a field, element or payload). It is
refused only on hole 3's wide reading. So the edit's ODR-075 line, 'closes the one route by which its result
reached C from safe code', is not true.
what_goes_wrong: A raw pointer that safe code chose flows to C through an Ember function's result, not through a C function's
input: J8-08's overflow by another route.
rule_lines: p16-ffi.md 400-405 (FFI-21), 424-434 (FFI-21a), 83 (FFI-10 as edited); p04-types.md 340-342
fix_hint: Treat an Ember function's result given to C as a value safe code hands to C (the same direction property as
hole 3).

## NG-18 (from deep/v3/results/test2.json)
kind: unsafe_allowed
caused_by: pre_existing
severity: high
Program:
```
// SDL_surface.h (SDL2, abridged)
typedef struct SDL_PixelFormat SDL_PixelFormat;
typedef struct SDL_Surface { Uint32 flags; SDL_PixelFormat *format; int w, h; int pitch; void *pixels; } SDL_Surface;
SDL_Surface* SDL_CreateRGBSurface(Uint32 flags, int width, int height, int depth, Uint32 Rmask, Uint32 Gmask, Uint32 Bmask, Uint32 Amask);
void SDL_FreeSurface(SDL_Surface *surface);   /* frees surface->pixels, then the surface */

# overlays/sdl_surface.em
unsafe overlay c "SDL_surface.h":
    @ffi(destroys(SDL_FreeSurface))
    type SDL_Surface
    @safety("`surface` was made by SDL and is not freed yet")
    @ffi(no_callbacks)
    unsafe fn SDL_FreeSurface(surface)
    @safety("the caller ends the result once, with `SDL_FreeSurface`")
    @ffi(no_callbacks)
    unsafe fn SDL_CreateRGBSurface(flags, width, height, depth, Rmask, Gmask, Bmask, Amask)

# main.em
from std.ffi import adopt
import c "SDL_surface.h" with (overlay = "overlays/sdl_surface.em") as sdl

fn new_canvas() -> owns sdl.SDL_Surface:
    # SAFETY: SDL made the surface, and nothing else owns or frees it
    unsafe:
        return adopt[sdl.SDL_Surface](sdl.SDL_CreateRGBSurface(0, 64, 64, 32, 0, 0, 0, 0))

fn main():
    canvas = new_canvas()
    stray: u8 = 0
    canvas.pixels = ref_to_ptr(ref mut stray)
    println(canvas.w)
```
today: Compiles. When `canvas` ends, the compiler calls `SDL_FreeSurface` (p16 386-389, 599-601), which frees
`pixels`, now a stack byte: heap corruption, caused by one line of safe code. The `unsafe:` note vouched for
the adoption, not for later writes.
with_edit: The same. The edit judges declarations, and the `returns_owned` result route is refused (rightly), but `adopt`
inside `unsafe:` is no declaration, and the field write stays safe.
what_goes_wrong: A raw pointer that safe code chose reaches C through memory C will read later (a destructor, or C's own copy),
not through any call's arguments. This is another route the edit's ODR-075 line says it closed.
rule_lines: p16-ffi.md 386-393 (FFI-36a), 551-561 and 599-601 (destructors); p04-types.md 340-346
fix_hint: Make writing a raw-pointer field of memory C will read (a foreign `owns T`, a `ref mut` view C gave) unsafe,
as writing through a raw pointer is.

## NG-19 (from deep/v3/results/test2.json)
kind: not_general
caused_by: pre_existing
severity: high
Program:
```
// arena.hpp
#include <cstddef>
#include <cstdint>
class Arena {
public:
    virtual ~Arena() {}
    virtual uint8_t* grab(std::size_t n) = 0;   // gives n writable bytes
};
void decode_frame(Arena& arena) noexcept;        // calls arena.grab(4096) and writes 4,096 decoded bytes there

# overlays/arena.em
# SAFETY: the entry below matches arena.hpp and its documentation
unsafe overlay cpp "arena.hpp":
    fn decode_frame(arena)

# main.em: a video player decodes a frame into scratch memory it supplies
import cpp "arena.hpp" with (overlay = "overlays/arena.em") as codec

class StackArena(codec.Arena):
    override fn grab(n: usize) -> *mut u8:
        one: u8 = 0
        return ref_to_ptr(ref mut one)      # safe code: a raw pointer keeps no view

fn main():
    arena = StackArena()
    codec.decode_frame(ref mut arena)       # C++ writes 4,096 bytes into grab's finished frame
    println("decoded")
```
today: Compiles. `Arena&` imports as `ref mut arena: Arena` (p21-annex-c.md 161-162). The entry asserts
`decode_frame` safe (p16-ffi.md 37-38, 278-280). The trampoline passes the override `grab` to C++
(p21-annex-c.md 121-125), and a borrowed parameter takes the Ember object upcast to its base (139-140).
Nothing judges the override's raw-pointer result, and `ref_to_ptr` of a local is safe (p04-types.md 340-345).
C++ writes through a dangling stack pointer: a use after return, caused from safe code.
with_edit: The same on the reading that compiles. `Arena` is an opaque class, so 'at any depth of an argument or the
object' finds no field, and the override reaches C++ through no callback parameter. 'What an Ember function
given to C returns' describes the override, but the words name no place to refuse it: refusing the
`decode_frame` entry would refuse it also for an `Arena` that C++ made.
what_goes_wrong: An Ember function that C++ calls hands it a raw pointer chosen by safe code, through a call asserted safe. The
edit's line under ODR-075 ('Their results no longer reach C through a call asserted safe') does not hold for
an override. It does not hold either for an `@unsafe(export(...))` function whose result the host writes
through, since that attribute vouches only for the symbol (p16-ffi.md 630-634).
rule_lines: p21-annex-c.md 121-125 (FFI-39), 139-140 (FFI-39c), 161-162 (FFI-40); p16-ffi.md 37-38 (FFI-2), 83 (FFI-10 as
edited); p04-types.md 340-345
fix_hint: An Ember function that foreign code can call never returns from safe code a raw pointer that C reads, however
C got the function (a callback, a field, an override, an export).

## NG-20 (from deep/v3/results/test2.json)
kind: undecided
caused_by: pre_existing
severity: high
Program:
```
// nethooks.h
#include <stddef.h>
#include <stdint.h>
struct net_hooks { uint8_t* (*grab)(size_t n); };
struct net_hooks* net_hooks_get(void);   /* the library's one hooks table, read by every later net_recv */
int net_recv(size_t n);                  /* writes n received bytes where hooks->grab(n) points */

# overlays/nethooks.em
# SAFETY: the entries below match nethooks.h and its documentation
unsafe overlay c "nethooks.h":
    @ffi(no_callbacks)
    fn net_hooks_get() -> borrowed one exclusive from(static)
    fn net_recv(n)

# main.em: a server gives the network library its own receive buffers
import c "nethooks.h" with (overlay = "overlays/nethooks.em") as net

fn scratch(n: usize) -> *mut u8:
    one: u8 = 0
    return ref_to_ptr(ref mut one)

fn main():
    hooks = net.net_hooks_get()
    hooks.grab = scratch          # safe code: an Ember function written into C's table
    println(net.net_recv(1500))   # C writes 1,500 bytes into scratch's finished frame
```
today: Compiles. `net_hooks_get` gives a `ref mut net_hooks` (exclusive, p16-ffi.md 294). The field `grab` imports as
`unsafe extern "C" fn(usize) -> *mut u8` (590-592). The capture-free `scratch` coerces to it
(p05-declarations.md 182-185). Neither entry has a raw-pointer parameter or result (82-84). C writes through a
dangling pointer.
with_edit: Undecided, and the same as today on one reading. The edit lists 'a result safe code may write (`ref mut`,
...)' and, separately, 'what an Ember function given to C returns'. Here the two combine: the Ember function
is given to C by a field write through the `ref mut` result. If 'at any depth' passes through a callable field
to its result, `net_hooks_get` is E5002. If it stops at data fields, nothing is refused and the program writes
through a dangling pointer as today.
what_goes_wrong: A raw pointer chosen by safe code reaches C by a route that is the composition of two routes the edit names,
and the words do not say whether the composition is reached. Two compilers disagree. On the reading that
compiles, a server's receive path corrupts the stack.
rule_lines: p16-ffi.md 83 (FFI-10 as edited), 290-294 (FFI-11), 590-592 (FFI-56), 451-458 (FFI-52); p05-declarations.md
182-185 (FN-6)
fix_hint: An Ember function counts as given to C wherever its value reaches memory C reads (an argument, a field of one,
a write through a `ref mut` or `MutSpan` C gave), and its result is judged there.

## NG-21 (from deep/v3/results/test2.json)
kind: not_general
caused_by: pre_existing
severity: high
Program:
```
# plugin.em: a game module the host loads; the host asks it for a scratch buffer and copies a packet into it
# SAFETY: the host defines no other symbol named `plugin_scratch`
@unsafe(export("plugin_scratch"))
pub extern "C" fn scratch(want: usize) -> *mut u8:
    byte: u8 = 0
    return ref_to_ptr(ref mut byte)      # safe: a raw pointer keeps no view

fn main():
    println("module ready")

// host.c, linked with the module:
//   uint8_t* buf = plugin_scratch(1500);
//   memcpy(buf, packet, 1500);           /* writes 1,500 bytes into scratch's finished frame */
```
today: Compiles. FFI-31b's list of what may not appear in an export does not include a raw pointer (p16-ffi.md
647-651). FFI-21a refuses only a pointer into a by-value parameter's copy, and `byte` is a local (424-434).
`ref_to_ptr(ref mut byte)` is safe (p04-types.md 340-342). The export's `# SAFETY:` note vouches for the
symbol only (622-636). The host writes through a dangling stack pointer: a use after return.
with_edit: Unchanged. "What an Ember function given to C returns" could be read to reach an export, but the refusal falls
only on "Every declaration asserting foreign code safe to call", and an export is no such declaration. So it
still compiles. The same holds for an `@export_table` field and for a C++ override that C++ calls
(p21-annex-c.md 121-125, 147-155).
what_goes_wrong: The edit's own direction property (a raw pointer that safe code made, read by C) has other routes into C that
no declaration covers: every way FFI-52 lists for C to reach an Ember function besides a declaration's
callback argument. This is a new gap for the list, not this edit's.
rule_lines: p16-ffi.md 83 (as edited), 424-434 (FFI-21a), 451-458 (FFI-52's list), 622-651 (FFI-26, FFI-31b);
p21-annex-c.md 121-125, 147-155; p04-types.md 340-342
fix_hint: Judge every Ember function C can call (export, table field, override, callback) by the one direction: a raw
pointer it returns to C that safe code made is refused where it is returned.

## NG-22 (from deep/v3/results/test2.json)
kind: unsafe_allowed
caused_by: pre_existing
severity: high
Program:
```
import std.thread
from std.sync import Receiver, channel

class Node:
    hits: int = 0

fn count(rx: Receiver[shares Node]) -> int:
    n = 0
    for node in rx:
        n += node.hits
    return n

fn main():
    tx, rx = channel[shares Node](capacity=4)
    h = thread.spawn(owns fn() owns rx: count(rx))
    root = shares Node()
    tx.send(owns shares root).expect("counter is gone")   # a shares owner goes to the other thread
    keep = shares root                                   # this thread adds to the count the other thread subtracts from
    println(keep.hits)
```
today: No rule refuses it. THR-15 p11 336-337 says `Sender` is Send and ThreadShared and `Receiver` is Send, whatever
the element, so the spawn holding `rx` passes THR-10. THR-2 p11 99-106 forbids a `shares` owner reaching
another thread 'by any route', but its list of routes names no channel, and it gives no line or code. This
also falsifies the 'today' claims of the server and renderer siblings, which rest on the lead fix J7-07.
with_edit: The same. The edit does not touch THR-15.
what_goes_wrong: The counter thread ends its received owner with a plain subtract while main's `keep = shares root` adds. The
count is lost, and a freed Node is read later (C++: a non-atomic refcount touched by two threads).
rule_lines: p11-concurrency.md 336-337 (THR-15), 99-106 (THR-2), 175-176 (THR-10)
fix_hint: A channel end is Send and ThreadShared exactly when its element is Send (the lead fix J7-07).

## NG-23 (from deep/v3/results/test2.json)
kind: undecided
caused_by: pre_existing
severity: low
Program:
```
import std.thread
from std.sync import Sender, channel

interface Command:
    fn run() -> int

struct Clear:
    color: int

extend Clear implements Command:
    fn run() -> int:
        return this.color

fn main():
    cmd: owns dyn Command + Send = owns Clear(color=3)
    println(cmd.run())
```
today: Undecided. TYP-5 rule 8 converts "a value to an interface its type implements", `owns T` to `owns dyn I` (p04
84-86): one interface. No rule says that `owns Clear` becomes `owns dyn Command + Send`, or what a `dyn` with
a `+` list may call.
with_edit: The same: the edit does not touch TYP-5. The edit's renderer sibling and the std code behind AnyError's From
(`Box(value)` into `Box[dyn Error + Send + ThreadShared]`, core.em 97-99 after the change) depend on this
conversion.
what_goes_wrong: The edit's own sibling program, and std's From, depend on a conversion the specification does not state. It
belongs on the gap list (TYP-5 and IV.9).
rule_lines: p04 71-86 (TYP-5); p04 779-784 (IV.9); p03 72, 171
fix_hint: A value converts to `dyn` of a list of interfaces when its type implements each one, and a `dyn` of a list may
be viewed as a `dyn` of any part of it.

## NG-24 (from deep/v3/results/test2.json)
kind: undecided
caused_by: pre_existing
severity: high
Program:
```
import std.thread
from std.sync import RwLock

class Texture:
    width: int = 0

class Assets:                              # a game engine's asset cache, shared by its loader threads
    loaded: RwLock[Array[Texture]] = RwLock([])

fn unload_all(assets: thread_shared Assets):
    with g = assets.loaded.write():
        g.clear()                          # ends every Texture, on this thread

fn main():
    assets = thread_shared Assets()
    hero = Texture(width=64)
    preview = weakref hero                 # a one-thread link: hero's weak count is plain
    with g = assets.loaded.write():
        g.push(owns hero)                  # hero is now reachable from every thread
    h = thread.spawn(owns fn() thread_shared assets: unload_all(assets))
    if preview exists as p:
        println(p.width)
    else:
        println("gone")
    h.join()
```
today: Compiles: Texture is Send and ThreadShared (THR-8 p11 47-48; THR-9 p11 63), so thread_shared Assets() passes
THR-1 (p11 82-85) and verdict 165. Verdict 114's hand-over (THR-2 p11 112-122) is stated for 'an object handed
whole to another thread'. RC-4 point 4 (p08 173-178) lists 'into a lock-guarded thread_shared container' as a
place other threads reach, but only for owners of thread_shared values. No rule says that `g.push(owns hero)`
moves hero away from `preview`.
with_edit: The same. The edit does not touch THR-2 or RC-4. It makes AnyError ThreadShared, so its own game sibling keeps
errors, and any class objects they own, in such a holder: the same route.
what_goes_wrong: If the store is not a hand-over, the worker's `g.clear()` ends hero and marks its header with plain writes
while main's `preview exists as p` tests that header and reads `p.width`. That is a data race on a one-thread
weak count and a read of an ended object (C++: a weak_ptr::lock racing the last shared_ptr's release on a
non-atomic control block). If it is a hand-over, main prints `gone`. The words do not decide which.
rule_lines: p11-concurrency.md 99-106, 112-122 (THR-2), 82-85 (THR-1); p08-classes.md 155-156, 173-178 (RC-4 points 1 and
4), 196-204 (RC-5 point 1), 346-350 (WK-12)
fix_hint: An object stored where another thread can reach it is handed over at that store, for its one-thread links as
for its counts, at every place RC-4 point 4 lists.

## NG-25 (from deep/v3/results/test2.json)
kind: unsafe_allowed
caused_by: pre_existing
severity: high
Program:
```
import std.thread
from std.fmt import Formatter, FmtError

class SaveSlot:
    index: int = 0

open class GameError:                      # a game's base error type
    message: String

    fn init(owns message: String):
        this.message = message

extend GameError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(this.message)

extend GameError implements Error:
    pass

class SaveError(GameError):                # the save system's error, keeping the slot it failed on
    slot: shares SaveSlot

    fn init(owns message: String, slot: shares SaveSlot):
        this.slot = shares slot
        super.init(owns message)

fn save(slot: shares SaveSlot) -> Result[int, GameError]:
    return Err(SaveError(owns String("disk full"), shares slot))   # upcast to GameError

fn main():
    slot = shares SaveSlot()
    r = save(slot)
    h = thread.spawn(owns fn() owns r: r.is_err())
    again = shares slot
    println(h.join(), again.index)
```
today: Compiles. Class objects are Send when every component is (THR-8 p11 47-48), and GameError's only field is a
String, so R = Result[int, GameError] is Send (THR-10 p11 176). TYP-5 rule 8 (p04 86) upcasts the SaveError,
and no rule says a derived class must keep its base's markers (CLS-4 p05 335-345 says nothing of them).
with_edit: The same for this program. The AnyError route (save returning `Result[int]`, with `?`) depends on whether a
class that gets Error through its base 'implements Error' under ERR-3's new E2040. The edit's words do not
say.
what_goes_wrong: The worker ends `r`, so the SaveError and its `slot` owner, with a plain subtract while main's `again = shares
slot` adds. The count is lost and the slot is later read after it is freed (C++: a derived exception holding a
non-atomic shared_ptr, caught by base reference on another thread).
rule_lines: p11-concurrency.md 43-51 (THR-8), 175-176 (THR-10); p04-types.md 84-86 (TYP-5 rule 8); p05-declarations.md
335-345 (CLS-4); p13-errors.md 88 (ERR-3 as edited)
fix_hint: A derived class keeps every thread marker its base has, refused at its declaration otherwise, so a type's
markers hold for every object that may stand behind it.

## NG-26 (from deep/v3/results/test2.json)
kind: needless_refusal
caused_by: pre_existing
severity: medium
Program:
```
// config.hpp (C++)
// namespace srv {
// struct Entry { int port; };
// class Config {
// public:
//     Config() noexcept;
//     const Entry& at(size_t i) const noexcept { return entries_[i]; }
//     Entry& at(size_t i) noexcept { return entries_[i]; }
// private:
//     Entry entries_[4];
// };
// }
import cpp "config.hpp" with (project="server") as net

fn main():
    # SAFETY: entry 0 is one of the four the config holds
    unsafe:
        cfg = net.srv.Config.new()
        e = cfg.at(0)
        println(e.port)
```
today: Refused: the two `at` are E1030 (TYP-26) or undecided.
with_edit: E2070, naming both. `cfg` is a local, a changeable name, so both `at` compile, the arguments are the same, and
the result is not compared. The edit says so and leaves it to K47.
what_goes_wrong: The most common C++ accessor pairs (`at`, `front`, `back`, `data`, `begin`) are refused through every
changeable name, which is the ordinary local. C++ runs the non-const `at`. The program goes wrong the same way
without the edit, so this is a gap for K47, not a hole in this edit. The tie-break K47 chooses will have to
amend FFI-39b's 'several fitting equally'.
rule_lines: FFI-39b added sentence; TYP-24 p04-types.md 563-566; IV.11 step 3 p04-types.md 860-863
fix_hint: K47: the object counts in the fit as an argument does, a changeable name fitting the declaration that may
change it as its own type and the read-only one through a coercion.

## NG-27 (from deep/v3/results/test2.json)
kind: needless_refusal
caused_by: pre_existing
severity: medium
Program:
```
// events.hpp (C++)
// namespace ui {
// struct KeyEvent { int code; };
// struct MouseEvent { int x; int y; };
// class Handler {
// public:
//     Handler() noexcept;
//     virtual ~Handler() noexcept;
//     virtual void on(const KeyEvent& e) noexcept;
//     virtual void on(const MouseEvent& e) noexcept;
// };
// void dispatch_key(Handler& h, int code) noexcept;          // calls h.on(KeyEvent{code})
// void dispatch_click(Handler& h, int x, int y) noexcept;    // calls h.on(MouseEvent{x, y})
// }
import cpp "events.hpp" with (project="ui") as ui

class InputLog(ui.ui.Handler):
    keys: int = 0
    clicks: int = 0

    fn init():
        # SAFETY: the base constructor sets no state Ember reads
        unsafe:
            super.init()

    override fn on(e: ui.ui.KeyEvent):
        this.keys += 1

    override fn on(e: ui.ui.MouseEvent):
        this.clicks += 1

fn main():
    log = InputLog()
    # SAFETY: the dispatchers call `on` on the handler during the call only
    unsafe:
        ui.ui.dispatch_key(ref mut log, 27)
        ui.ui.dispatch_click(ref mut log, 10, 20)
    println(log.keys, log.clicks)
```
today: E1030 (TYP-26): the base's two `on` and the class's two `on`.
with_edit: E1030 on the class's second `on`. TYP-26's added sentence covers only the header's declarations ('the header,
not the program, declares them'), so it does not reach overrides.
what_goes_wrong: Overriding every overload of a C++ virtual, which is the visitor and event-handler pattern, is refused. The
program means to print `1 1`. It is refused the same way without the edit, so this is pre-existing. But the
edit's reason for its exception also holds here: the Ember class must write one override per C++ overload.
rule_lines: TYP-26 p04-types.md 867-868 and its added sentence; CLS-4 p05-declarations.md 335-338; FFI-39 p21-annex-c.md
121-125
fix_hint: An Ember class may write one override per inherited C++ declaration of a name, each replacing the declaration
it fits, since the header, not the program, made them several.

## NG-28 (from deep/v3/results/test2.json)
kind: undecided
caused_by: pre_existing
severity: low
Program:
```
// stream.hpp (C++)
// namespace io {
// class Out {
// public:
//     Out() noexcept;
//     void put(int v) noexcept;
//     void put(long v) noexcept;
//     void put(long long v) noexcept;
// private:
//     int written_;
// };
// }
import cpp "stream.hpp" with (project="tools") as s

fn main():
    n: c_int = 5
    # SAFETY: `put` only appends to the stream's buffer
    unsafe:
        out = s.io.Out.new()
        out.put(n)
```
today: Refused: the three `put` are E1030, or undecided.
with_edit: FFI-8 (p16-ffi.md 173) maps `int`, `long` and `long long` to `c_int`, `c_long` and `i64`, but does not say
whether `c_int` and `c_long` are distinct types or aliases. On MSVC both are 32-bit. If they are aliases,
`put(int)` and `put(long)` import with one signature, `out.put(n)` is E2070, and no typed entry can name
either.
what_goes_wrong: On one target, stream-like `int`/`long` overload sets become impossible to call or vouch for, while C++ runs
`put(int)`. The edit lists this in to_verify. It rests on the type identity FFI-8 leaves open, so it is
pre-existing.
rule_lines: FFI-8 p16-ffi.md 172-173
fix_hint: Each C integer type the importer names (`c_int`, `c_long`, `c_char`) is a type of its own, as C's are,
converting to its fixed-size twin by widening rules only.

## NG-29 (from deep/v3/results/test2.json)
kind: crash_or_undefined
caused_by: pre_existing
severity: high
Program:
```
// jobs.hpp (C++)
// namespace jobs {
// class Payload {
// public:
//     explicit Payload(int size) noexcept;   // allocates `size` bytes
//     Payload(Payload&& other) noexcept;     // takes other's buffer, leaves it empty
//     ~Payload() noexcept;                   // frees the buffer it holds
//     int size() const noexcept { return size_; }
// private:
//     int size_;
//     char* buffer_;
// };
// class Sink {
// public:
//     Sink() noexcept;
//     virtual ~Sink() noexcept;
//     virtual void take(Payload&& p) noexcept {}
// };
// void feed(Sink& s, int size) noexcept;   // { Payload p(size); s.take(std::move(p)); }  ~Payload() of p at its end
// }
# overlays/jobs.em
unsafe overlay cpp "jobs.hpp":
    class Payload:
        @ffi(const)
        fn size()

# src/main.em
from std.ffi import c_int
import cpp "jobs.hpp" with (project="engine", overlay="overlays/jobs.em") as jb

class Counter(jb.jobs.Sink):
    bytes: c_int = 0

    fn init():
        # SAFETY: the base constructor sets no state Ember reads
        unsafe:
            super.init()

    override fn take(owns p: jb.jobs.Payload):
        this.bytes += p.size()

fn main():
    c = Counter()
    # SAFETY: `feed` calls `take` on the sink during the call only
    unsafe:
        jb.jobs.feed(ref mut c, 64)
    println(c.bytes)
```
today: Compiles under the words. FFI-40 imports `T&&` as `owns x: T` (p21-annex-c.md 161-163), so the override writes
`owns p`. FFI-39 passes the virtual to it (p21-annex-c.md 121-125). An `owns` parameter is the function's own
and ends at its end, which runs the class's destructor on 'an object of its own' (FFI-32, p21-annex-c.md
77-80). Nothing says the trampoline first moves C++'s object into one Ember owns.
with_edit: The same: `take` is one function, so the added sentence does not apply.
what_goes_wrong: Read plainly, Ember ends `p` at the end of `take`: it runs ~Payload and frees a block that is `feed`'s stack
local. `feed` then runs ~Payload on it again: a double free of `buffer_`, and a free of a stack address. In
C++ terms, a callee deleting an object it only received by rvalue reference. The call-side twin
(`cache.add(owns splash)` to `add(Texture&&)`) leaves the moved-from object's end unsaid too, which risks a
leak, not unsafety. This is a new gap for the list, in the edit's reach but not caused by it.
rule_lines: FFI-40 p21-annex-c.md 161-163; FFI-39 p21-annex-c.md 121-125; FFI-32 p21-annex-c.md 74-80; FN-2a
p05-declarations.md 115-131
fix_hint: An object C++ lends through a reference stays C++'s to end: an `owns` parameter fed from a `T&&` receives an
object of its own, made by the move constructor.

## NG-30 (from deep/v3/results/test2.json)
kind: unsafe_allowed
caused_by: pre_existing
severity: medium
Program:
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
// class FocusEvent : public Event {
// public:
//     FocusEvent() noexcept;
// };
// class Handler {
// public:
//     Handler() noexcept;
//     virtual ~Handler() noexcept;
//     virtual void on(const Event& e) noexcept {}
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
today: Compiles under the words, or undecided. CLS-4: 'A method named like an inherited one replaces it'
(p05-declarations.md 337). DSP-2: 'an override reuses the base's slot' (p08-classes.md 287). FFI-39 passes the
virtual to the override (p21-annex-c.md 121-125). No rule compares an override's parameter types with those of
the method it replaces, for an Ember base or a C++ one. E5056 ('override of a C++ virtual the overlay does not
name', p17a-codes.md 188) has no rule text left.
with_edit: The same: `on` is one function, so the added sentence does not apply.
what_goes_wrong: The same type confusion as H1, through a single virtual: a FocusEvent reaches Ember code that reads it as a
KeyEvent. A careful implementer checks the signature, which is why this is medium. Two levels show the same
cause: under CLS-4's by-name rule, `class B(A): override fn on(e: MouseEvent)` replaces A's `override fn on(e:
KeyEvent)`. This is a new gap for the list.
rule_lines: CLS-4 p05-declarations.md 335-339; DSP-2 p08-classes.md 286-287; FFI-39 p21-annex-c.md 121-125; p17a-codes.md
188
fix_hint: An override's parameter types, words and result are those of the method it replaces, exactly; anything else is
refused, naming both.

## NG-31 (from deep/v3/results/test2.json)
kind: needless_refusal
caused_by: pre_existing
severity: medium
Program:
```
// events.hpp (C++)
// namespace ui {
// struct KeyEvent { int code; };
// struct MouseEvent { int x; int y; };
// class Handler {
// public:
//     Handler() noexcept;
//     virtual ~Handler() noexcept;
//     virtual void on(const KeyEvent& e) noexcept;
//     virtual void on(const MouseEvent& e) noexcept;
// };
// void dispatch_key(Handler& h, int code) noexcept;          // calls h.on(KeyEvent{code})
// void dispatch_click(Handler& h, int x, int y) noexcept;    // calls h.on(MouseEvent{x, y})
// }
import cpp "events.hpp" with (project="ui") as gui

class InputLog(gui.ui.Handler):
    keys: int = 0
    clicks: int = 0

    fn init():
        # SAFETY: the base constructor sets no state Ember reads
        unsafe:
            super.init()

    override fn on(e: gui.ui.KeyEvent):
        this.keys += 1

    override fn on(e: gui.ui.MouseEvent):
        this.clicks += 1

fn main():
    log = InputLog()
    # SAFETY: the dispatchers call `on` on the handler during the call only
    unsafe:
        gui.ui.dispatch_key(ref mut log, 27)
        gui.ui.dispatch_click(ref mut log, 10, 20)
    println(log.keys, log.clicks)
```
today: E1030 (TYP-26 p04-types.md 867-868), for the base's two `on` and for the class's two `on`.
with_edit: E1030 on the class's second `on`. TYP-26's added sentence covers only 'A C++ name of several functions ... the
header, not the program, declares them', and InputLog's methods are the program's.
what_goes_wrong: Re-test of E1-11: confirmed pre_existing, because the class's own two methods are refused the same way with or
without the edit. The visitor and event-handler pattern (overriding every overload of a C++ virtual) stays
impossible, and the program means to print `1 1`. The edit's reason (the header forces the names) holds here
too, so this is a gap for the list.
rule_lines: TYP-26 p04-types.md 867-868 and its added sentence; CLS-4 p05-declarations.md 335-339; FFI-39 p21-annex-c.md
121-125
fix_hint: An Ember class may write one override per inherited C++ function of a name, each replacing the one it fits,
since the header made them several.

## NG-32 (from deep/v4/results/pilot.json)
id: S1-1
role: safety
round: 1
block: none
kind: unsafe_allowed
severity: high
caused_by_change: false
Program:
```
# stream.c (linked): void stream_read(size_t want, void (*get_buf)(size_t n, uint8_t **out));
# it calls get_buf(want, &p), then writes `want` bytes at p
unsafe extern "C":  # SAFETY: the declaration matches stream.c
    @ffi(param(get_buf, callback(borrowed)))
    safe fn stream_read(want: usize, get_buf: extern "C" fn(usize, ref mut *mut u8))

fn spare(n: usize, ref mut out: *mut u8):
    x: u8 = 0
    out = ref_to_ptr(ref mut x)

fn main():
    stream_read(4096, spare)
    println("read")
```
walk: Line 5. FFI-10 (p16-ffi.md 59-61) accepts a `safe fn` when 'every parameter and result is a scalar, a
`@layout(c)` value type, or a pointer whose contract is complete'. `want` is a scalar. `get_buf` is a function
pointer contracted `callback(borrowed)` (FFI-21, p16 400-401: 'A C function-pointer parameter accepts a
capture-free Ember function'). Its callable type is legal: FN-6 (p05-declarations.md 175-176) says 'A callable
type is written `fn(A, ref mut B, owns C) -> R`, with the same parameter words as a declaration', and
(185-186) 'A function that captures nothing also coerces to `extern "C" fn(…)` when every type is FFI-safe'.
tests/conformance/FFI-21/accept_native_callback_adapters.em line 78 already uses `extern "C" fn(ref mut Pair)
-> i32`.
The changed FFI-2 (p16 38) has three parts, and none reaches this line. (1) 'a raw pointer ... that a call
hands C inside what an argument or a method's object passes': the argument is the function `spare`, which
holds no raw pointer. (2) 'nor one that a function the call gives C returns, itself or inside its result':
`spare` returns nothing. (3) 'a raw pointer C hands Ember (a result, an `out` value, a callback's parameter)
is not one': `out` is the slot C hands `spare`. So there is no E5002, and line 5 stands.
Line 9. IV.5 (p04-types.md 340-345): 'Creating one from a reference is safe ... The raw pointer keeps no
compiler-tracked view'. E3063 (FN-3, p05 151-152, and p04 330-334, 'a view of the callee's own locals ... put
there is `E3063`') reaches views only, so writing this raw pointer through the `ref mut` parameter is ordinary
safe code.
Line 12. C calls `spare(4096, &p)`, receives the address of `x`, whose frame has ended, and writes 4,096 bytes
there.
Without the change, the same list accepts line 5 and the same write happens, so caused_by_change is false.
impact: Memory corruption from safe code. In C++ terms: a callback hands back the address of a dead local through
`uint8_t** out`, then C makes a 4 KB write into the stack. It is the danger of Decision 2's own `stream_read`
program, by the other way C callbacks hand a buffer back. The chosen option ii says 'everything the call hands
C', and its owner_short recommendation says it 'covers every way a pointer made in safe code reaches C'. Both
are untrue for this program. The C++ form is the same: an override `fn get_buf(n: usize, ref mut out: *mut
u8)` of `virtual void get_buf(size_t, uint8_t*&)`.
suggestion: Property: no assertion of the fact covers a raw pointer that a function the call gives C hands back to C,
whether as its result or by writing it through a pointer C passes it (a `ref mut` parameter, at any depth).
The carve-out then covers only what C hands Ember and Ember reads, and the C++ `uint8_t*&` override form also
needs objection 2's override reach. Alternative location for the designer to weigh: FFI-21a (p16 424-434)
already limits the raw-pointer result of a function C runs, at the point where its value becomes `extern "C"
fn`, following the pointer 'through copies, conversions and aggregates'. One sentence there ('a raw pointer
made in safe code that such a function hands C, by its result or through a parameter C gave it, is made in
unsafe code') reaches this program, objection 2's override, objection 3's field function and exports alike. It
also keeps `stream_read` safe for callbacks that hand back a pointer C gave them.

## NG-33 (from deep/v4/results/pilot.json)
id: S1-2
role: safety
round: 1
block: none
kind: unsafe_allowed
severity: high
caused_by_change: false
Program:
```
// buffers.hpp
#pragma once
#include <cstddef>
#include <cstdint>

class Source {
public:
    Source() noexcept {}
    virtual ~Source() {}
    virtual std::uint8_t* get_buf(std::size_t n) noexcept { return nullptr; }   // must give n writable bytes
};

inline void fill_zero(Source& src, std::size_t n) noexcept {   // writes n zeros at src.get_buf(n)
    std::uint8_t* p = src.get_buf(n);
    for (std::size_t i = 0; i < n; i++) p[i] = 0;
}

# overlays/buffers.em
unsafe overlay cpp "buffers.hpp":  # SAFETY(ffi): the entry matches its declaration in the header
    fn fill_zero(src, n)

# main.em
import cpp "buffers.hpp" with (overlay = "overlays/buffers.em")

class Spare(cpp.Source):
    override fn get_buf(n: usize) -> *mut u8:
        x: u8 = 0
        return ref_to_ptr(ref mut x)

fn main():
    s = Spare()
    cpp.fill_zero(ref mut s, 4096)
    println("filled")
```
walk: main.em line 3. FFI-39 (p21-annex-c.md 121-124): 'An Ember class may derive from a C++ class with no overlay
entry ... the importer generates a C++ subclass, the trampoline, which passes on to the Ember class exactly
the virtual methods it overrides'. ODR-221 removed `virtuals=[…]` (p26 1368). Line 6 returns a raw pointer
made in safe code (IV.5, p04 340-342).
overlays/buffers.em line 2. GRM-35 (p16 278-280) says a function entry 'asserts the function safe to call'.
FFI-1 (p16 26-27) asks for facts only 'for each pointer-typed parameter and result'. `src` is `Source&`, which
FFI-40 (p21 161) imports as `ref mut src: Source`, and `n` is a scalar, so every fact is known.
The changed FFI-2 (p16 38) does not reach the entry. (1) `Source` has no raw-pointer field. (2) 'a function
the call gives C returns' names a function, but the call gives C++ an object, and the override reaches C++
through the object's virtual table. The spec itself counts overrides as a separate way C runs Ember code:
FFI-52 (p16 455-456) lists 'every override of an Ember class derived from a C++ class whose objects C++ may
hold' apart from 'every Ember function given to C as an `extern "C" fn` value' (457-458).
main.em line 10 is safe code. FFI-39c (p21 139-140) says 'a parameter borrowed for the call alone takes the
object ... upcast to the C++ base'; tests/conformance/FFI-17d calls `cpp.keep(Leaf())` from safe code the same
way. C++ calls `Spare.get_buf(4096)`, receives the address of a dead local and writes 4,096 zeros there.
With `fn keep(node: owns)` (FFI-17d's overlay), C++ keeps the object, and a later call with no arguments runs
the override, when no call of the change's kind gives C anything.
Today the same, so caused_by_change is false.
impact: Memory corruption from safe code, in C++ terms an override returning the address of a local. C++ frameworks
commonly ask an overridable source, allocator or stream for a buffer through a virtual method, and every Ember
subclass of such a class is reached. The change's words, its tests and its to_verify list cover callback
parameters only.
suggestion: Property: the reach of 'a function the call gives C' is every Ember function the call lets C run because of
what it passes: function values at any depth in it, and the overrides of the class objects in it (FFI-52's own
list). A call that hands C++ an object of a class whose virtual returns a raw pointer, and that Ember may
derive from, is under the limit. Or place the limit at the override itself, as objection 1's alternative does
at FFI-21a's location, which also avoids refusing every safe entry that takes such a class.

## NG-34 (from deep/v4/results/pilot.json)
id: S1-3
role: safety
round: 1
block: none
kind: unsafe_allowed
severity: medium
caused_by_change: false
Program:
```
# hooks.c (linked): `set_hooks` keeps `hooks`; `scratch(n)` calls `hooks.alloc(n)` and writes `n` bytes at the result
@layout(c)
struct Hooks:
    alloc: extern "C" fn(usize) -> *mut u8
    release: extern "C" fn(*mut u8)

unsafe extern "C":  # SAFETY: the declarations match hooks.c
    @ffi(no_callbacks)
    safe fn set_hooks(hooks: Hooks)
    safe fn scratch(n: usize)

fn spare(n: usize) -> *mut u8:
    x: u8 = 0
    return ref_to_ptr(ref mut x)

fn drop_buf(p: *mut u8):
    pass

fn main():
    set_hooks(Hooks(alloc=spare, release=drop_buf))
    scratch(4096)
    println("parsed")
```
walk: Line 9. `Hooks` is two function pointers, and IV.5 (p04 349-350) says '`extern "C" fn(i32) -> i32` is a C
function pointer: `Copy`, no captures'. So `Hooks` is 'a `@layout(c)` value type' in FFI-10's list (p16
59-61), and the declaration is accepted. Line 10 is a scalar `safe fn` with no `no_callbacks`, and FFI-52 (p16
457-458) counts 'every Ember function given to C as an `extern "C" fn` value' among what it may run.
The changed FFI-2 (p16 38). (1) 'a raw pointer (`*T`, `*mut T`) ... inside what an argument ... passes ... at
any depth (a field, an element)': `Hooks` holds function pointers, not `*T`. (2) 'nor one that a function the
call gives C returns': `spare` is given to C inside a field of the argument, but the 'at any depth' of (1) is
not said of (2). The designer's only sibling, test and to_verify for (2) are a callback parameter (`get_buf`).
So one reader refuses line 9 (E5002 naming `hooks.alloc`'s result), and another accepts it.
Under the second reading, line 21 runs `spare(4096)` from C, which receives the address of a dead local and
writes 4,096 bytes there. Today the program compiles, with the same result, so caused_by_change is false.
impact: This is the allocator-hook pattern, the shape of cJSON's `cJSON_InitHooks(cJSON_Hooks*)` with `void
*(*malloc_fn)(size_t)`: a library keeps a struct of functions and calls them during later calls. Memory
corruption from safe code under one reading of the words; two compilers built from them disagree.
suggestion: Say the reach of (2) as (1)'s: 'a function the call gives C, as an argument or at any depth in one' (or fold
it into objection 1's property). Add this program as a must-refuse test.

## NG-35 (from deep/v4/results/pilot.json)
id: S1-4
role: safety
round: 1
block: none
kind: unsafe_allowed
severity: medium
caused_by_change: false
Program:
```
// dsp.hpp
#pragma once
#include <cstddef>
#include <functional>
using RenderFn = std::function<void(float* out, std::size_t frames)>;
RenderFn make_silence() noexcept;    // the result writes `frames` zeros at `out`

# overlays/dsp.em
unsafe overlay cpp "dsp.hpp":  # SAFETY(ffi): the entries match their declarations in the header
    @ffi(std_function)
    type RenderFn
    @ffi(no_callbacks)
    fn make_silence()

# main.em
import cpp "dsp.hpp" with (overlay = "overlays/dsp.em") as dsp

fn main():
    x: f32 = 1.0
    render = dsp.make_silence()
    render(ref_to_ptr(ref mut x), 4096)
    println(x)
```
walk: overlays/dsp.em lines 2-3. Annex C (p21-annex-c.md 67): with `@ffi(std_function)` on its overlay entry, a
`std::function` is 'a function value of type `fn(A…) -> R` (`[CLO-3]`), called as any other'. Here that is
`fn(*mut f32, usize)` (FFI-8, p16 180), a safe callable type, not `unsafe fn(…)` (FN-6, p05 179-183).
Today nothing fences the call. FFI-10's raw-pointer refusal (p16 82-84) is for hand declarations only. FFI-2's
unknown-fact rule (p16 33-34) has no slot here, since a `std::function` value's parameters take no contract
words. FFI-56 (p16 590-598) makes a C function pointer from C unsafe to call unless `@ffi(safe)` asserts it
within its list, but it speaks of C function pointers only.
The change's response says `@ffi(std_function)` 'is none of the three assertions the words name ... so its
import is unchanged'. Yet the added sentence begins 'No assertion of the fact covers…', and its 'an overlay
entry not written `unsafe fn` ... whose call would hand C one' literally includes this type entry when a
parameter is a struct holding a raw pointer. So the struct case is decided two ways. The direct raw-pointer
case of this program is not 'inside' an argument, and no rule reaches it.
main.em line 6 is safe code. `ref_to_ptr(ref mut x)` is safe (IV.5), and C++ writes 4,096 floats over the
4-byte `x`. Today the same, so caused_by_change is false.
impact: A stack buffer overflow from safe code through any `std::function` that C++ hands Ember with a raw pointer, or
a struct holding one, among its parameters: render callbacks, filters, allocator hooks. FFI-56 fences the same
kind of value when it comes from C as a function pointer, but not when it comes from C++ as a `std::function`.
suggestion: Give a `std::function` that C++ hands Ember FFI-56's property: a function from foreign code is unsafe to call
unless asserted safe, within FFI-10's list and FFI-2's limit. The callable type it would then be already
exists (`unsafe fn(A…) -> R`, FN-6, ODR-115), so no new callable syntax is needed. Name `@ffi(std_function)`
among the assertions the added sentence reaches. By Decision 3's parameter, the entry is then refused where it
is written, with a line to write: the unsafe spelling of the type entry. That spelling is the one missing
piece, and checklist 12 says to add it.

## NG-36 (from deep/v4/results/pilot.json)
id: S1-5
role: safety
round: 1
block: none
kind: impact_report_wrong
severity: low
caused_by_change: false
Program:
```
// engine.h
#include <stdint.h>
typedef struct frame_input { float delta_time; const uint8_t *ini_path; uint32_t ini_len; } frame_input;
frame_input *engine_input(void);    /* the engine's input block, alive for the whole program */
void engine_begin_frame(void);      /* reads the block; saves settings to the `ini_len` bytes at `ini_path` */

# overlays/engine.em
# SAFETY: the entries below match engine.h
unsafe overlay c "engine.h":
    @ffi(no_callbacks)
    fn engine_input() -> borrowed one exclusive from(static)
    @ffi(no_callbacks)
    fn engine_begin_frame()

# main.em
import c "engine.h" with (overlay = "overlays/engine.em") as en

fn set_settings_file():
    name: Array[u8] = [97, 46, 105, 110, 105]
    input = en.engine_input()
    input.ini_path = name.as_span().as_ptr()
    input.ini_len = 5

fn main():
    set_settings_file()
    en.engine_begin_frame()
    println("frame")
```
walk: overlays/engine.em line 4. FFI-11 (p16 293-294) contracts the result as a `ref mut` view of C's block ('`ref
mut` and `MutSpan[T]` results allowed only with `exclusive`'; `from(static)`).
main.em line 6. SPN-8 (p07-ownership.md 837-838): '`as_ptr()` ... extracting one is safe'. Writing a field
through a `ref mut` view is safe code.
The changed FFI-2 (p16 38) reaches neither entry. `engine_input()`'s result is covered by 'a raw pointer C
hands Ember (a result ...) is not one, since reading through it is `unsafe` already', and
`engine_begin_frame()` hands C nothing. Here safe code reads nothing: it writes its own pointer into C's
memory.
main.em line 11. C reads 5 bytes from `name`, which was freed when `set_settings_file` returned: a
use-after-free (in C++ terms, a dangling `const char*` stored in a global config). Today the same, so
caused_by_change is false.
impact: The designer lists this route in notes (gap 1). But why_general ('a raw pointer that safe code made never
reaches C under an assertion of safety'), Decision 2's owner_short recommendation ('covers every way a pointer
made in safe code reaches C') and the carve-out's stated reason tell the owner it is closed. Objections 1-4
are further routes. The owner would weigh Decision 2 on a wrong account.
suggestion: State in the carve-out that it covers what C hands Ember and Ember only reads. Say in owner_short and
why_general which routes stay open: writes into memory C holds through a view it hands Ember (here, ImGui's IO
block), a destructor reading fields safe code wrote, and those of objections 1-4. List the field-write route
for the owner as one question with this program.

## NG-37 (from deep/v4/results/pilot.json)
id: S1-6
role: safety
round: 1
block: none
kind: unsafe_allowed
severity: low
caused_by_change: false
Program:
```
// text.hpp
#pragma once
#include <cstddef>
namespace txt {
struct Slice {
    const char* p;
    std::size_t n;
};
std::size_t count_spaces(const Slice& s) noexcept;   // counts ' ' in the n bytes at p
}

# overlays/text.em
unsafe overlay cpp "text.hpp":  # SAFETY(ffi): the entry matches its declaration in the header
    @ffi(no_callbacks)
    fn count_spaces(s)

# main.em
from std.ffi import c_char
import cpp "text.hpp" with (overlay = "overlays/text.em") as t

fn main():
    s = t.txt.Slice(p=null[*c_char](), n=64)
    println(t.txt.count_spaces(s))
```
walk: overlays/text.em line 3. FFI-40 (p21 161-162): '`const T&` as a plain parameter `x: T`'. It is no pointer, so
FFI-1 (p16 26-27) asks no fact of it, and the entry asserts the call safe (GRM-35, p16 278-280).
The changed FFI-2 (p16 38) reaches 'inside what an argument or a method's object passes, by value or behind a
pointer'. A C++ reference is neither by value (C++ passes the address) nor 'behind a pointer' in the spec's
own words, which keep references and pointers apart (FFI-36, p21 118: 'A C++ function returning a raw
pointer'; FFI-40's reference rows). The designer's C++ sibling covers only 'a method's object'. So one reader
refuses line 3 (naming `s.p`), and another accepts it.
Under the second reading, main.em line 6 has C++ read 64 bytes at null: a fault with no stop at an Ember line.
Today the same, so caused_by_change is false.
impact: Low, since the change's intent clearly covers this program. But every C++ API taking a struct of pointer and
length by `const&` (string-view-like slices, buffer descriptors) depends on how the word 'pointer' is read.
suggestion: Say the reach by what the call lets C read: 'however the call passes it (by value, through a pointer or a C++
reference)'. Add this program as a must-refuse test.

## NG-38 (from deep/v4/results/pilot.json)
id: E1-7
role: engineering
round: 1
block: none
kind: not_general
severity: low
caused_by_change: false
Program:
```
@layout(c)
struct Reply:
    text: *u8
    len: usize

# SAFETY: the host defines no other symbol named `plugin_reply`
@unsafe(export("plugin_reply"))
pub extern "C" fn reply() -> Reply:
    buf: Array[u8] = [111, 107]
    return Reply(text=buf.as_span().as_ptr(), len=2)

fn main():
    println("built")
```
walk: 1. The change covers "one that a function the call gives C returns, itself or inside its result". Its stated
property is "a raw pointer that safe code made never reaches C under an assertion of safety".
2. An exported function is a function C calls whose result reaches C by the same route as a callback's result.
FFI-26 (p16 631-634) asks for a `# SAFETY:` note "saying why nothing else defines that name", which covers the
name only.
3. FFI-31b (p16 651) lets "`@layout(c)` values" appear in an exported signature.
4. FFI-21a's E5042 (p16 428-433) refuses only a raw pointer "into such a parameter", not one into a local.
5. So `reply` compiles today and after the change. The host reads 2 bytes of a freed Array (C++: returning a
pointer into a destroyed `std::vector`). `@export_table` fields (p16 623-628) work the same way.
impact: This is an existing gap, not caused by the change. But the change covers callback results and leaves exports,
the same route, so the property holds for one kind of 'function C calls' and not the other.
suggestion: Cover every function C may call with one property: one a call gives C, an export, an export table's field. The
export already carries a vouch (`@unsafe(export)`), so the cheapest fix is to widen what its `# SAFETY:` note
covers: "and, when its result holds a raw pointer, why that pointer is valid for C". Or list it for later with
the change's other existing gaps.

## NG-39 (from deep/v4/results/pilot.json)
id: S2-1
role: safety
round: 2
block: none
kind: not_general
severity: medium
caused_by_change: false
Program:
```
// timer.h
typedef void (*tick_fn)(int ms);
#define TICK_ROM_HANDLER ((tick_fn)0x1FFF0000)   /* the boot ROM's tick handler on the board this header targets */
typedef struct timer_spec { tick_fn fire; int ms; } timer_spec;
void timer_run(timer_spec spec);    /* calls spec.fire(spec.ms) */

# overlays/timer.em
# SAFETY: the entry below matches timer.h
unsafe overlay c "timer.h":
    fn timer_run(spec)

# main.em
import c "timer.h" with (overlay = "overlays/timer.em") as tm

fn main():
    spec = tm.timer_spec(fire=tm.TICK_ROM_HANDLER, ms=16)
    tm.timer_run(spec)
    println("ran")
```
walk: 1. `timer_spec` is a complete struct, so it imports as a `@layout(c) struct` (FFI-8, p16 182). Its field
`fire` is 'a function pointer that comes from C — a field of an imported struct' and 'imports as `unsafe
extern "C" fn(…)`' (FFI-56, p16 590-592; FFI-8 row, p16 188).
2. `TICK_ROM_HANDLER` is a pointer-cast macro. It is 'a `const` of that pointer type holding the address as an
integer' (FFI-6a, p16 148-151), so its type is `unsafe extern "C" fn(c_int)`.
3. main.em line 4 builds the struct holding that value in safe code. UNS-1 (p09 266-269) puts only calling
'through a value of an `unsafe` callable type' in unsafe, not holding or storing one.
4. overlays/timer.em line 3 is a function entry, 'which asserts the function safe to call' (GRM-35, p16
278-280). `spec` is a `@layout(c)` value type with no pointer slot, so no fact is unknown (FFI-2, p16 33-35).
5. The added limit covers 'a raw pointer (`*T`, `*mut T`) that the caller's arguments carry to foreign code
... or what an Ember function given to the call ... returns'. `fire` is not `*T` or `*mut T`, and no Ember
function is given. The entry stands, line 5 is safe code, and C calls address 0x1FFF0000 (C++: calling through
a function pointer cast from an integer).
6. If the importer gives the field `Option[…]` (p16 188: '`Option[…]` when nullable'), `fire=None` hands C a
null function pointer from safe code in the same way.
7. Today the same program compiles and fails the same way, so the change does not cause it. But the change's
own reason lists 'holds a constant address (`[FFI-6a]`)', which is equally true of a function-pointer
constant. Its why_general also claims the limit 'reaches every type that carries a pointer'.
impact: A forged or null C function pointer still reaches C from safe code through a struct argument, which is the
same hole J8-08 closes for data pointers. The structs involved are common: hook tables, ops structs and
allocator callback structs. The claim that 'every pointer the call's arguments carry to C' is covered does not
hold.
suggestion: State the limit by the property, not by a list of types. No assertion covers a pointer value the caller
supplies whose target nothing checks. That means a raw pointer, and also a C function pointer value (`unsafe
extern "C" fn(…)`, or an `Option` of one), which safe code can hold from a constant or from C. A safe `extern
"C" fn` made from an Ember function is checked, and only what it hands back is reached. Then `timer_run` is
E5002 naming `spec.fire`, and the binding builds the struct from an Ember function inside a wrapper's
`unsafe:` block.

## NG-40 (from deep/v4/results/pilot.json)
id: S2-2
role: safety
round: 2
block: none
kind: not_general
severity: medium
caused_by_change: false
Program:
```
// labels.hpp
#pragma once
#include <vector>
namespace ui {
std::vector<const char*> default_names() noexcept;               // returns {"ok"}
void print_all(const std::vector<const char*>& names) noexcept;   // std::puts each name
}

# overlays/labels.em
# SAFETY: the entries below match labels.hpp
unsafe overlay cpp "labels.hpp":
    @ffi(no_callbacks)
    fn default_names()
    @ffi(no_callbacks)
    fn print_all(names)

# main.em (ember.toml has a [cpp.ui] section)
from std.ffi import c_char
import cpp "labels.hpp" with (project = "ui", overlay = "overlays/labels.em") as u

fn main():
    names = u.ui.default_names()
    names.push_back(null[*c_char]())
    u.ui.print_all(names)
    println("printed")
```
walk: 1. `std::vector<T>` imports as '`CppVector[T]` with `.span()`, `.span_mut()`, `.to_array()`, `push_back`,
`len`' (FFI-17a, p21 62). With `T` = `const char*`, that is `*c_char` (FFI-8, p16 180).
2. A `const T&` parameter imports 'as a plain parameter `x: T`' (FFI-40, p21 161-162), so `print_all(names:
CppVector[*c_char])`. The entry asserts the call safe (GRM-35, p16 278-280).
3. main.em line 6 is safe code. `null[*c_char]()` 'is safe' (IV.5, p04 347), and `push_back` is a bridge
operation, not one of UNS-1's unsafe operations (p09 266-273).
4. The added reach is 'a field or an element, at any depth, of what an argument passes ..., as its Ember type
declares it'. The change's why_general reads this as 'opaque classes, extern types and bridge types, which
declare no fields, are not reached', and its Unchanged row lists `CppVector`. So the element safe code pushed
is not reached, `print_all` stands, line 7 is safe code, and C++ calls `std::puts(nullptr)` (C++:
dereferencing a null C string).
5. If 'an element ... as its Ember type declares it' is read to reach a `CppVector`'s elements instead, the
impact report is wrong. Either way the words do not decide it.
6. The same holds for any wrapper whose pointer safe code chooses but whose Ember type declares no fields:
`Atomic[*mut T]` from `_Atomic(T*)` (p16 192), `Volatile[*T]` (UNS-5, p09 302), `ObjectPtr[C]` copies (FFI-23,
p16 548-550).
7. Today the program compiles and fails the same way, so the change does not cause it.
impact: The carve-out separates 'not reached' from 'reached' by whether a type declares fields. The question that
matters for safety is whether safe code could have set the pointer. With that line, every `std::vector<const
char*>`, `std::vector<iovec>`-style or `std::vector<Slice>` parameter of a safe C++ entry hands C++ pointers
chosen by safe code.
suggestion: Draw the exclusion by who sets the pointer, not by whether fields are declared. The limit reaches every raw
pointer in what an argument passes that safe code could have set: a field, an element, or an element of a
container or wrapper whose type argument is or holds a raw pointer (`CppVector[T]`, `Atomic[T]`, `Option[T]`).
It does not reach what only foreign code or the compiler sets, such as an opaque class's members or
`CppString`'s buffer. Then `print_all` is E5002 naming the elements of `names`, and the Mesh test still prints
`3`.

## NG-41 (from deep/v4/results/pilot.json)
id: S2-3
role: safety
round: 2
block: none
kind: not_general
severity: low
caused_by_change: false
Program:
```
@layout(c)
struct BufDesc:
    data: *mut u8      # the decoder writes `len` bytes here
    len: usize

unsafe extern "C":
    @ffi(param(get_buf, callback(borrowed)))
    safe fn decode_frame(get_buf: extern "C" fn(ref mut BufDesc))

fn fill(ref mut d: BufDesc):
    x: u8 = 0
    d.data = ref_to_ptr(ref mut x)
    d.len = 4096

fn main():
    decode_frame(fill)
    println("decoded")
```
walk: 1. FN-6 (p05 175-176): 'A callable type is written `fn(A, ref mut B, owns C) -> R`, with the same parameter
words as a declaration'. FN-6 (185-186): 'A function that captures nothing also coerces to `extern "C" fn(…)`
when every type is FFI-safe'. `BufDesc` is `@layout(c)`. FFI-21 (p16 400-401) accepts 'a capture-free Ember
function (as an `extern "C" fn`)'.
2. In `fill`, line 12 makes a raw pointer from a local, which is safe (IV.5, p04 340-342). Storing it in
`d.data` stores a pointer value and writes through no raw pointer, so it is not in UNS-1 (p09 266-267). 'The
raw pointer keeps no compiler-tracked view' (p04 345), so E3063 does not apply.
3. The added reach is 'what an Ember function given to the call ... returns, itself or in its result'. `fill`
returns nothing: it hands C the pointer through its `ref mut` parameter. `decode_frame` stands, and C writes
4,096 bytes into `fill`'s dead stack slot (C++: an out-parameter left pointing at a dead local, as with
FFmpeg's `get_buffer2` filling `AVFrame.data[]`).
4. Assumption: this holds only if an `extern "C" fn` type may carry `ref mut`. FN-6 does not forbid it, and
the specification does not say either way.
5. Today the program compiles and fails the same way, so the change does not cause it.
impact: Callbacks that fill a descriptor C passes in (buffer providers, allocator hooks that write their answer into a
struct) hand C pointers chosen by safe code. Decision 2's 'covers every pointer a call hands C' does not hold
for this route.
suggestion: State the callback route by direction rather than by result. The limit reaches every raw pointer an Ember
function given to the call hands back to C: what it returns, and what it leaves in a place C passed it (`ref
mut` or `out` parameters). Then `decode_frame` is E5002 naming what `get_buf` leaves in `BufDesc.data`.

## NG-42 (from deep/v4/results/pilot.json)
id: S1-2
role: safety
round: 1
block: none
kind: unsafe_allowed
severity: high
caused_by_change: false
Program:
```
import std.thread
from std.sync import Mutex
from std.fmt import Formatter, FmtError

class LoadError:
    path: String = String("level1.cfg")

extend LoadError implements Display:
    fn fmt(ref mut f: Formatter) -> Result[void, FmtError]:
        return f.write_str(this.path)

extend LoadError implements Error:
    pass

class Log:
    results: Mutex[Array[Result[int]]]

fn fail_typed(owns e: LoadError) -> Result[int, LoadError]:
    return Err(e)

fn fail(owns e: LoadError) -> Result[int]:
    return Ok(fail_typed(owns e)?)                   # the LoadError object goes into an AnyError's box

fn drain(log: thread_shared Log):
    with g = log.results.lock():
        g.clear()                                    # ends the LoadError object on this thread

fn main():
    e = LoadError()
    w = weakref e                                    # a link on this thread: plain weak count
    log = thread_shared Log(results=Mutex([fail(owns e)]))
    h = thread.spawn(owns fn() thread_shared log: drain(log))
    if w exists as p:
        println(p.path)                              # may read the String drain frees
    h.join()
    println("done")
```
walk: The danger in C++ terms: a weak_ptr's control block is updated without atomics while another thread destroys
the object it points at.
The change relies on verdict 114 for class errors (cost: "THR-2's existing hand-over check ... reaches objects
the boxed error owns ... through the error's table"). THR-2 (p11 112-114) runs that check only when "An object
handed whole to another thread leaves its `weakref` links behind ... At the hand-over the program reads the
object's weak count".
Line 31, `thread_shared Log(...)`. THR-1 (p11 69-72) allows thread_shared before a new value. Its check that
"everything reachable from it is `Send`" (p11 82) passes, because AnyError is Send after the change (p11 51;
p04 676 after). Verdict 165's ThreadShared check passes, because THR-9 (p11 66) makes "`Mutex[T]` for a `Send`
`T`" ThreadShared. Nothing is handed to a thread at this line: the object is moved into a new value. So
THR-2's move does not run, and `w` still points at the LoadError object, which the worker can now reach
through the Mutex.
Line 32: the worker gets one more owner of `log` (THR-17, p11 186). On the worker, `g.clear()` (line 26) ends
the LoadError object: OBJ-3 (p08 83-86) runs its field drops, which free `path`'s String, and decrements its
weak count. RC-4 (p08 155-156) says that count is changed with plain adds and subtracts.
Lines 33-34 on main: WK-12 (p08 346-348) says opening the link "tests the target's header". RC-5 point 1 (p08
196-204) checks only lines and calls of this thread. RC-5 point 2 (p08 215) counts only for "A view of a
`thread_shared` object". So main tests the header and reads `p.path` while the worker ends the object and
frees the String: a data race on the weak count, and a read after free.
Not caused by the change: the same program with `class Enemy` in a `Mutex[Array[Enemy]]` (Enemy is Send by its
fields, p11 47-48) compiles today and goes wrong the same way. The change routes the default error type
through this route (its game sibling is exactly this shape) while counting on verdict 114's check, which does
not run here.
impact: A use-after-free reached by storing a result into a lock of a thread_shared object while a link on this thread
points at the error object. It is the same for any class object in a thread_shared container today.
suggestion: This is a gap for the list, in THR-2, not a reason to block this change. The property: an object becomes
reachable from another thread by any route THR-2 already lists (handed over, sent, or stored into a
thread_shared value or a lock of one), and by any such route it leaves its weakref links behind, as verdict
114's hand-over does: one compare at that line. With that, line 31 moves the object, `w` finds it ended, and
the program prints `done`. Also correct the change's cost line, which currently implies verdict 114 covers
every route a boxed error takes to another thread.

## NG-43 (from deep/v4/results/pilot.json)
id: S1-4
role: safety
round: 1
block: none
kind: impact_report_wrong
severity: low
caused_by_change: false
Program:
```
import std.thread
from std.sync import Sender, Receiver, channel

class Doc:
    path: String = String("main.em")

fn consume(owns rx: Receiver[shares Doc]) -> int:
    n = 0
    for d in rx:
        n += d.path.len()
    return n

fn send_all(owns tx: Sender[shares Doc], doc: shares Doc):
    for i in 0..1000:
        tx.send(owns shares doc).expect("receiver is gone")

fn main():
    tx, rx = channel[shares Doc](capacity=4)
    h = thread.spawn(owns fn() owns rx: consume(rx))
    doc = shares Doc()
    send_all(owns tx, doc)
    println(h.join())
```
walk: The danger in C++ terms: a non-atomic refcount (an Rc-like shared_ptr) pushed through a thread-safe queue to
another thread.
The change's impact gives, for its server program, "before: E7015 at the spawn line (the Sender holds
AnyErrors, which are not Send)", and its sibling relies on "a Sender is Send when its element is". The text
does not say that. THR-15 (p11 336-337) says "`Sender` is `Send`, `ThreadShared` and `Clone` ... `Receiver` is
`Send`", with no condition on the element.
So in this program, line 19 hands `rx` (a Receiver, Send by p11 337) to a thread. Each `shares doc` sent on
line 15 is a one-thread owner, which the worker ends with a plain -1 (RC-4, p08 155-156) while main makes the
next owner with a plain +1: a data race on the count, and then a read after free of `d.path`. This compiles
today. It is J7-07, the group's open lead fix, and it exists without the change. The change does close it for
AnyError, by accident, since AnyError becomes Send.
impact: The change's 'before' column claims a refusal where today's text has a hole. The owner would read the server
case as safe today when it is not. The change does not add any risk here.
suggestion: Correct the server row's 'before' to: "compiles by THR-15's words (Sender and Receiver are Send whatever the
element); the J7-07 fix (a channel end is Send only for a Send element) is what makes it E7015". Keep J7-07 on
the list.

## NG-44 (from deep/v4/results/pilot.json)
id: I1-5
role: integration
round: 1
block: none
kind: impact_report_wrong
severity: low
caused_by_change: false
Program:
```
import std.thread
from std.sync import Sender, channel

fn parse_port(text: StrView) -> Result[int]:
    match text.parse[int]():
        Ok(n):
            return Ok(n)
        Err(_):
            return Err(AnyError.msg(f"not a port: {text}"))

fn produce(tx: Sender[Result[int]]):
    for text in ["8080", "http"]:
        tx.send(owns parse_port(text)).expect("receiver is gone")

fn main():
    tx, rx = channel[Result[int]](capacity=4)
    producer = thread.spawn(owns fn() owns tx: produce(tx))
    for r in rx:
        match r:
            Ok(port):
                println(port)
            Err(e):
                println(f"error: {e}")
    producer.join()
```
walk: Three statements in the report are not what the text says. (1) Server sibling, before: the report gives "E7015
at the spawn line (the Sender holds AnyErrors, which are not Send)", and its walk quotes "a Sender is Send
when its element is". THR-15 (p11 335-336) instead says "`Sender` is `Send`, `ThreadShared` and `Clone`" with
no condition. By THR-15 the spawn line holds a Send `tx` and compiles today. By THR-8 (p11 44-48) it does not.
That clash exists already (J7-07). After the change both rules accept this program. (2) Generic-library
sibling: R is `Result[int, AnyError]` whatever `S.Failure` is (ERR-9 p13 48-49). So what decides it is p11 51
through AnyError's box, not "IFC-3 carries Send and ThreadShared into `S.Failure`". (3) Cost: "which the
compiler knows from the From instances". An `owns dyn Error` is also made outside AnyError's From (a wrapper's
`cause: owns dyn Error` field, an `@from` variant's `source`). The condition for verdict 114's check to run is
that some type implementing Error owns an object of a class that `weakref` links point at.
impact: The before/after table and the cost line that the owner reads quote words the specification does not have. No
outcome after the change is wrong.
suggestion: Server before: write 'THR-15 says it compiles and THR-8 says E7015 (J7-07, a clash that exists already)'.
Generic library: cite p11 51 through AnyError. Cost: state the condition as 'a type implementing Error owns an
object whose class a weakref is made to'.

## NG-45 (from deep/v4/results/pilot.json)
id: I1-6
role: integration
round: 1
block: none
kind: other
severity: low
caused_by_change: false
Program:
```
import std.thread

fn parse_level(text: StrView) -> Result[int]:
    return Ok(text.trim().parse[int]()?)

fn main() -> Result[void]:
    h = thread.spawn(owns fn() => parse_level("sixty-four"))
    n = h.join()?
    println(n)
    return Ok(())
```
walk: The change's tests k20_worker_failure_reaches_main and k20_server_channel_of_results expect `the text is not a
value of this type`. That text is std/src/string.em line 22, not the specification. p15-stdlib.md 158-159 says
only "`ParseError`, in `std.string`, is a unit-only enum, `Empty`, `Invalid` or `Overflow`". By DRV-1 p14 130,
a derived Display of a unit-only enum is "the variant name". So the specification decides neither text, and
the expected stderr is not one the specification fixes (rule E: "an error code the specification names ... or
the exact text the program prints").
impact: Two of the change's tests follow the implementation, not the specification (memory rule: tests follow the
spec). If std's text changes, they fail with nothing in the specification having changed.
suggestion: Make the failure come from text the specification fixes: `AnyError.msg(f"not a level: {text}")` (ERR-8 p13
98-100), or the test's own `@derive(Error)` enum with an `@error(...)` text (ERR-3). See the replacement tests
below.

## NG-46 (from deep/v4/results/pilot.json)
id: I1-5
role: integration
round: 1
block: none
kind: not_general
severity: medium
caused_by_change: false
Program:
```
import std.thread
from std.sync import Atomic, Receiver, Sender, channel

static FREED: Atomic[int] = Atomic(0)

class Buffer:                                 # a network buffer the reader thread filled
    id: int
    fn drop():
        FREED.fetch_add(1)

class Packet:                                 # a plain message carrying the buffer to main
    buf: thread_shared Buffer

fn reader(tx: Sender[Packet], stop: Receiver[int]) -> int:
    tx.send(owns Packet(buf=thread_shared Buffer(id=1))).expect("main is gone")
    return stop.recv().unwrap()               # the reader waits here for main's word to stop

fn show(owns p: Packet):
    println(p.buf.id)                         # `p` ends here: the Buffer goes back to the reader's queue

fn main():
    tx, rx = channel[Packet](capacity=4)
    stop_tx, stop_rx = channel[int](capacity=1)
    h = thread.spawn(owns fn() owns tx, owns stop_rx: reader(tx, stop_rx))
    show(owns rx.recv().unwrap())
    while FREED.load() < 1:                   # danger in C++ terms: a deleter owed to a thread blocked in a read (a leak, here a hang)
        pass
    stop_tx.send(owns 0).expect("the reader is gone")
    println(h.join())
    println(FREED.load(), "buffer freed")
```
walk: 1. The Buffer travels inside a Packet, so it is not handed over (RC-4 point 5, p08 179-180: "it travelled
inside another object"). When main ends `p` in `show`, the Buffer goes into the reader's queue (point 2).
2. The reader's last check point is "before every call that may block" (changed point 3, kept from p08
169-170), that is, before `stop.recv()`. If the reader is already blocked in it when the Buffer is queued, the
next point comes only after `recv` returns.
3. The changed point 3 keeps a waiting thread at a check point, and wakes it, only for "a job worker waiting
for work". A `thread.spawn` thread blocked in a receive, a lock wait, a join or a sleep is not covered.
4. Main spins until FREED is 1, and the reader waits for main: the program may never end. Today it is the
same, so the change does not cause this. But its bound ('never left undone because that thread is asleep')
holds for one kind of wait only.
impact: A reader, logger or accept loop that made values and then blocks for a long time holds them that long,
possibly for ever. The change's own bound argument (J7-02's 'leaves resources held for ever') applies to them
unchanged.
suggestion: One property in point 3 instead of the worker case: a thread waiting in a call that may block (for work, a
lock, a channel's receive, a join, a sleep) stays at a check point while it waits, where one may be placed,
and is woken to empty its queue when a value is put in it. The job worker's wait is then one instance of it.
With that, this program prints 1, 0, '1 buffer freed'.

## NG-47 (from deep/v4/results/pilot.json)
id: E2-2
role: engineering
round: 2
block: none
kind: special_case
severity: medium
caused_by_change: false
Program:
```
import std.thread
from std.sync import Atomic

static CLOSED: Atomic[int] = Atomic(0)

class Session:                                # one player's session, made on main
    id: int
    fn drop():
        CLOSED.fetch_add(1)                   # in a server: give the player's slot back

class Batch:                                  # a plain object, handed to the server thread whole
    sessions: Array[thread_shared Session] = []

fn serve(owns b: Batch) -> int:
    n = b.sessions.len()
    b.sessions.clear()                        # the Session goes back to main's queue; main is waiting in join
    while CLOSED.load() < 1:                  # danger in C++ terms: a deleter owed to a thread blocked in join (it never runs)
        pass
    return n

fn main():
    b = Batch(sessions=[thread_shared Session(id=1)])
    h = thread.spawn(owns fn() owns b: serve(owns b))
    println(h.join(), CLOSED.load())
```
walk: 1. Main makes the Session (RC-4 point 1, p08 159-161). It travels inside `b`, so it is not handed over (point
5, p08 179-181).
2. Main reaches `h.join()`. The changed point 3 places a point before it ('before every call that may block
the thread (... `join` ...)'), since main holds nothing. At that point main's queue is still empty.
3. The server clears its list, and point 2 (p08 163-168) puts the Session in main's queue. Main is now inside
the join. The changed point 3 wakes only 'a job worker waiting for work', so nothing runs main's queue,
`CLOSED` stays 0, the server never returns, and the join never returns.
4. The same happens to a hand-written worker pool: a `thread.spawn` thread looping `for id in requests:` that
makes Textures and sends them inside a plain Bundle. When requests stop, the Textures main let go of wait in
the loader's queue for ever. That is J7-02's leak with `thread.spawn` in place of `std.jobs`.
5. Decision 2 rejected option C for job workers ('C leaves a correct program not doing what it says (principle
7)'), and the impact says 'The new points bound every kind of thread: a spawned thread, main, a job worker,
and a thread C calls in on'. For a spawned thread or main waiting in a blocking call, neither holds. The leak
exists today too, so it is not caused by the change. But the change bounds one kind of waiting thread and not
the others (OWNER-DIRECTION checklist 7 and 13).
impact: An ordinary server, one whose main spawns the server thread and joins it, and every hand-written worker pool
keep the resources the other threads let go of for as long as the thread waits: a session slot, a socket, GPU
memory. Here the program never ends. The impact report's claim that every kind of thread is bounded is untrue.
suggestion: Make the idle worker one case of a property: 'a thread waiting in a blocking call at a check point that is
placed there (waiting for work, a lock, a `join`, a receive, a sleep) is woken by a value put in its queue,
empties its queue there, and waits again.' This is safe because the place was already judged for that call
when the point before the wait was placed. A wait inside a place that keeps points out has no point, so
nothing wakes it. Cost: one wake on the ending thread's out-of-line path, as for a worker. Ember's blocking
calls wait through one per-thread parker that the queue can also wake (to measure). Record the limit that
remains: a thread blocked inside a foreign call, as in decision 4. Under this, the program prints `1 1`.

