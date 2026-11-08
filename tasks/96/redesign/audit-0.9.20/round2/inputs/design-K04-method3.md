## K04-weakref-outlives-arena (P02; J5-02): **holds**, 4 agents

Outcome: `decided`. Intent: The camera follows the level's boss with a link that does not keep it alive, and says "gone" once the boss is no more.
The rule that decides it: LT-4 (tasks/spec-0.9.9/parts/p07-ownership.md 613-615 (with p08-classes.md 328 [WK-11] and 85 [OBJ-3])): "LT-4, p07 613-615: 'A value made in a memory manager's instance (`owns Mesh(points=[0.0, 1.0]) using current`, `[ALC-1]`), and every value that holds one, lives no longer than that instance, wherever it is kept'. WK-11 (p08 328) uses the same verb for a link and its target: 'A link may hold no target' (and WK-12, p08 349: 'the link holds nothing'). OBJ-3 (p08 85): 'the block is freed when the weak count reaches 0', so a link keeps the block, which is memory the instance gave out. Verdict 24 states the property: 'no value outlives the arena it came from'. Verdict 107 says 'the checks that no value outlives its manager or is used after a reset run when compiling'. So the property already reaches a link. The words do not say so plainly, because a link does not keep its target alive. ALC-5 (p09 242-243) guards a reset only 'While a value made in a memory manager's instance lives', which misses a block that is still kept after its value ends."

### Edit: LT-4 (change, tasks/spec-0.9.9/parts/p07-ownership.md 614-615)

**Before:**

> and every value that holds one, lives no longer than that instance, wherever it is kept

**After:**

> and every value that holds one or keeps its block (a weak count keeps the block after the value ends, `[OBJ-3]`), lives no longer than that instance, wherever it is kept

Why general: The property: whatever keeps memory an instance gave out lives no longer than that instance. It is stated for the block, not for one construct. So it reaches every holder of a weak count (a link in a field, in a list, in a returned struct, in a static, in a captured state) and every target kind (a class object, a `shares` or `thread_shared` value). It reaches every manager kind: under one that frees nothing one by one, and under a FoundByAddress one, the last link now always ends while the instance lives. The check is LT-4's own E3063 when compiling, so nothing new runs.

### Edit: ALC-5 (change, tasks/spec-0.9.9/parts/p09-memory.md 242-243)

**Before:**

> While a value made in a memory manager's instance lives, the instance is changed only by what allocates from it

**After:**

> While a value made in a memory manager's instance lives, or its block is kept after it ends (`[OBJ-3]`), the instance is changed only by what allocates from it

Why general: The property: an instance is not rewound, cleared or reused while any block it gave out is still kept. Today it holds only while the value lives. It now also holds while a weak count keeps an ended value's block. This reaches every change that is not an allocation or a give-back (`reset()`, `clear`, a program's own rewinding method), through any name, in every manager kind. The block's own give-back, when the last link ends, is still allowed: it is the value's end, which gives a block back. It is ALC-5's own E3021 when compiling.

Readability: Unchanged, word for word: the ordinary level-arena program with a camera link compiles and reads as it does today, printing 500, then gone. No new word, annotation or ceremony. Only a program whose link would outlive its arena, or whose arena is reset under a kept block, is refused, and its message gives the line to write.
Cost: none: decided when compiling. These are the same lifetime check (E3063, LT-4) and reset check (E3021, ALC-5) the compiler already runs for values made in an instance, now applied to the holders of weak counts as well. Nothing runs, and no byte is added to any object or link. C++ pays nothing either, and its version is undefined behaviour. The weak count's existing costs (one compare per +1, one header test per opening, WK-11 and WK-12) are unchanged.

New gaps found on the way (not this edit's): 4

Tests rejected by the script, not written (1):
* mixed_manager_link_list: the expected output is a description, not the exact text: , with `a`'s block back to the built-in manager and `b`'s to `level`, and no sanitizer or 

To verify on the compiler:
* J5-02's example: today it compiles, and opening `cam.target` in `main` reads freed memory (show it under a sanitizer, or with an arena whose end poisons its buffer). After the edit it is E3063 at line 10, `cam.target = weakref boss`.
* The option-A version (arena made in `main` above `cam`, passed as `ref mut level`) compiles and prints 500, then gone, with no sanitizer report. `cam` ends before `level`, as DRP-2 orders the locals.
* With `cam = Camera()` written above `level = Arena.with_capacity(...)` in `main`, the same call is E3063: the link would outlive `level`.
* The server sibling: E3063 at `ACTIVE.push(weakref s)`; it compiles when the session is made without `using`.
* The parser sibling compiles and prints 1, both before and after the edit (no needless refusal for links held by values made in the same instance).
* The renderer sibling: `hud.focus = weakref s` inside `draw_frame` is accepted through the summary of the two `ref mut` parameters, checked at the call in `main` (`hud` is made after `frame`). `frame.reset()` is E3021. With `hud.focus = None` above the reset it compiles, which shows that the compiler's tracking is flow-sensitive for a link that has been cleared. If it is not, report the exact fix the message would have to give.
* The game sibling: E3063 at `return t`.
* The generated code for every accepted program is identical before and after the edit (no run-time cost added).
* A link to a value held `shares` made in an arena, and to a `thread_shared` value made in one, are bounded the same way (E3063 when kept past the instance).

