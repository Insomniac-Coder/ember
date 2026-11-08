# K46-ffi-handle-meaning

Question: What does @ffi(handle) make a C type, and does a handle parameter need pointer words?
Severity: medium; impact rank 46
Members: J8-14; primary J8-14
Why together: Single question.
Critic's notes: It edits the same XVI.4 Vulkan overlay as K07. Rewrite that example once, after both answers.

## Finding J8-14
Rules: FFI-11, FFI-11a, FFI-1, FFI-31b; file tasks/spec-0.9.9/parts/p16-ffi.md lines 232-245, 301-302, 310-312; kind undecided_case; severity medium
Quote from Hardened_11:
ok=X, error=N)` on the enum, `[FFI-16]`); `handle`; `callback(borrowed | retained | once, user_data=p)`

The finder's question:
What does `@ffi(handle)` on a type do, and does the spec's own overlay compile? `vkCreateBuffer(device:
borrowed, ...)` and `vkCmdSetViewport(commandBuffer: borrowed, ...)` give pointer contracts with no count
word, which FFI-11a makes E5012.

Program:
```
unsafe overlay c "vulkan/vulkan.h":
    @ffi(handle)
    type VkDevice
    @ffi(no_callbacks)
    fn vkDestroyDevice(device: borrowed, pAllocator: nullable borrowed one)   # `device: borrowed` with no count: E5012?

# what is `vk.VkDevice` in Ember: a Copy value? passed by value? may it be `owns`?
```

The checker's verdict: confirmed (route owner_odr, severity medium, confidence 0.65)
Reason:
`handle` is listed as a contract word (FFI-11, p16-ffi.md 302) and placed on types and results (FFI-1, 31-32),
and the glossary (p24-appx-f.md 27) calls it "a token that names something elsewhere". No rule gives its
meaning: what Ember type it produces, whether it is `Copy`, or whether a parameter of a handle type is a
pointer contract. The spec's own overlay writes `vkCreateBuffer(device: borrowed, ...)` and
`vkCmdSetViewport(commandBuffer: borrowed, ...)` (239-245) with no count word, which FFI-11a (310) makes E5012
unless `handle` changes that. In Vulkan, `VkDevice` is a pointer typedef, and the non-dispatchable handles
such as `VkBuffer` are 64-bit integers on 32-bit targets. No verdict or ODR (ODR-247 only refuses `handle` on
a function) defines it.

options:
[{'letter': 'A', 'summary': "`@ffi(handle)` makes the type an opaque `Copy` value with the C type's
representation (a pointer or a number). It is passed and returned by value and never dereferenced, and a
parameter of a handle type takes no pointer words.", 'code': 'unsafe overlay c "vulkan/vulkan.h":\n
@ffi(handle)\n    type VkDevice\n    @ffi(handle)\n    type VkBuffer\n    @ffi(no_callbacks)\n    fn
vkDestroyBuffer(device, buffer, pAllocator: nullable borrowed one)', 'outcome': 'Compiles, and `copy_of_device
= device` copies the handle. `device: borrowed` on a handle parameter is E5002, "a handle is passed by value
and takes no pointer words". The XVI.4 example drops `borrowed` on `device` and `commandBuffer`.'}, {'letter':
'B', 'summary': '`@ffi(handle)` is an opaque type behind a pointer. Its parameters keep pointer contracts, a
count included.', 'code': '    fn vkDestroyBuffer(device: borrowed one, buffer, pAllocator: nullable borrowed
one)', 'outcome': 'Compiles once every handle parameter says `borrowed one`. A handle is not `Copy`, and
`VkBuffer` (a number on 32-bit targets) cannot be described this way.'}]

recommendation:
{'letter': 'A', 'reason': "It matches what a handle is in C and in the glossary (a token naming something
elsewhere, passed by value), it covers handles that are numbers, and it makes the specification's own Vulkan
overlay compile with one rule.", 'checklist': '2, 5 (one meaning for `handle` whatever its C representation),
7, 15 (the word says what the type is)'}

