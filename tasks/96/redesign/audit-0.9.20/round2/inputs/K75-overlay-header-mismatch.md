# K75-overlay-header-mismatch

Question: What happens when an overlay's first line names a different header from the import?
Severity: low; impact rank 75
Members: J8-15; primary J8-15
Why together: Single question.
Critic's notes: 

## Finding J8-15
Rules: GRM-35, FFI-55, FFI-30c, FFI-12; file tasks/spec-0.9.9/parts/p16-ffi.md lines 254, 273-275, 336-339; kind verdict_missing; severity low
Quote from Hardened_11:
overlay_decl  := ["unsafe"] "overlay" ("c" | "cpp") STRING
  (`[FFI-12]`). Ember records the header each overlay was written for in `ember.overlays`,
  never in the overlay (`[FFI-55]`).

The finder's question:
Which header is an overlay for: the STRING its first line names, the `header =` its `ember.overlays` line
records, or the `import c` that names it? What happens when they differ?

Program:
```
# overlays/sdl.em
unsafe overlay c "SDL.h":
    @ffi(no_callbacks)
    fn SDL_Quit()

# main.em
import c "SDL2/SDL_video.h" with (overlay = "overlays/sdl.em") as sdl   # STRING says SDL.h: refused, applied, or ignored?
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.5)
Reason:
Verdict 121 moved the header's fingerprint out of the overlay line into `ember.overlays` ("The overlay line's
`with (header_hash = ...)` ... goes"). GRM-35 (p16-ffi.md 254) still has every overlay name a header (`overlay
("c" | "cpp") STRING`), FFI-17 (p21-annex-c.md 12-13) writes `overlay cpp "Header.hpp":`, and the `import c
... with (overlay = ...)` line names a header too. GRM-35 (274-275) also says the header is recorded "never in
the overlay". FFI-30c (222-223) applies another package's overlay "only to the header it was written for".
With three places naming the header, nothing says which one wins or what happens when the overlay's line names
a header other than the one imported.

options:
[{'letter': 'A', 'summary': "The overlay's first line says which header it describes. It must name the header
the import names (and the record's), and a different one is an error.", 'code': 'import c "SDL.h" with
(overlay = "overlays/sdl.em") as sdl', 'outcome': 'This compiles. The example is E5010, "`overlays/sdl.em` is
written for `SDL.h`, and this import is of `SDL2/SDL_video.h`".'}, {'letter': 'B', 'summary': "The overlay's
first line names no header, `unsafe overlay c:`. The import names it and the record keeps it.", 'code': '#
overlays/sdl.em\nunsafe overlay c:\n    @ffi(no_callbacks)\n    safe fn SDL_Quit()', 'outcome': "The example's
import applies the overlay to `SDL2/SDL_video.h`, and the record's fingerprints catch any declaration that
differs."}]

recommendation:
{'letter': 'A', 'reason': 'The overlay file keeps saying, in readable words, which header it is about, and a
mismatch is refused instead of silently applied. Only the fingerprint stays out of the file, as decided.',
'checklist': '2 (written meaning stays in the line), 8 (a mismatch is refused), 9, 10 (does not reopen the
decided fingerprint)'}

