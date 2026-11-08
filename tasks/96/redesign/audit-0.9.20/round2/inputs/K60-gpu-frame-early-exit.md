# K60-gpu-frame-early-exit

Question: What happens when a GPU frame ends without end_frame, on an error?
Severity: low; impact rank 60
Members: J7-19; primary J7-19
Why together: Single question.
Critic's notes: 

## Finding J7-19
Rules: GPU-2, GPU-3, THR-6; file tasks/spec-0.9.9/parts/p22-annex-d.md lines 27-30; kind undecided_case; severity low
Quote from Hardened_11:
one owner and is not `Copy`, and `end_frame(owns frame: Frame)` takes it over (`device.end_frame(owns frame)`
/ ends the name `frame`), so ending a frame that never began, or ending one twice, is refused when /
compiling.

The finder's question:
What happens when a `Frame` ends without `end_frame`, from an early `return` or `?`: is that refused when
compiling, or does the `Frame`'s own `drop` submit, discard, or stop the program?

Program:
```
fn render(ref mut dev: Device, scene: Scene) -> Result[void, GpuError]:
    if dev.begin_frame() exists as frame:
        tex = dev.create_texture(scene.desc)?     # `?` returns: `frame` ends here without end_frame
        draw_all(ref mut frame, scene, tex)
        dev.end_frame(owns frame)
    return Ok(())
```

The checker's verdict: confirmed (route owner_odr, severity low, confidence 0.55)
Reason:
GPU-2 (p22 27-30) settles a frame that never began and a frame ended twice, but not a frame that ends without
`end_frame`, from an early `return` or `?`. `Frame` is not `@must_drop` (THR-6, p11 350: only `Scope` and
`JobScope`), and Annex D gives `Frame` no `drop`, so the device's state after such an exit (a swapchain image
acquired, a recording open) is undecided. Searched all of Annex D, THR-6, ALC-1 and GPU-3.

options:
[{'letter': 'A', 'summary': "A Frame's own drop closes the frame and submits nothing", 'code': "# the program
above, unchanged\n# `Frame` has a drop: it ends the frame as `end_frame` does (gives the swapchain image back,
frees\n# the frame's memory) but discards the recording", 'outcome': 'Compiles. On the error nothing is drawn
this frame, the error is returned, and the next `begin_frame` works as usual.'}, {'letter': 'B', 'summary':
'Ending a frame without end_frame stops the program', 'code': '# the program above, unchanged\n# `Frame`\'s
drop stops the program: "panic: a Frame ended without end_frame (render, line 8)"', 'outcome': 'Compiles. On
the error the program stops at the `?` line, naming the frame.'}]

recommendation:
{'letter': 'A', 'reason': 'It is how every one-owner resource in the language behaves on an early exit (the
C++ RAII guard, `std::lock_guard`): the `?` stays an ordinary error path, and the device is left in a usable
state. B turns a handled error into a crash.', 'checklist': '1 (the error is returned, as the program says), 2
(accepts the correct program), 7 (no ceiling: every early exit), 13 (an ordinary drop, no special path)'}

