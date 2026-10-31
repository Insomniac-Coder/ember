---

# Annex D — GPU Host Model (`std.gpu`)

`std.gpu` is a library for driving a GPU from Ember — handles, frames, command recording, deferred
destruction and shader interfaces — over an abstract device that a host renderer implements through
the FFI, or that an Ember backend implements directly. It adds nothing to the language. Ember does not
compile shaders in this version.

```ember,fragment
type TextureHandle = Handle[gpu.Texture]
type BufferHandle = Handle[gpu.Buffer]

interface Device:
    fn begin_frame() -> Option[Frame]                   # None: skip this frame; no end_frame
    fn end_frame(owns frame: Frame)
    fn create_texture(desc: TextureDesc) -> Result[TextureHandle, GpuError]
    fn create_buffer(desc: BufferDesc) -> Result[BufferHandle, GpuError]
    fn destroy(h: TextureHandle)                        # physical destruction is deferred
    fn frames_in_flight() -> int
```

* `[GPU-1]` *(changed in 0.9.9)* GPU objects are named by generational handles (`[HND-1]`), which are
  `Copy` and `Send`. A stale handle is detected by its generation **in every profile**
  (`panic: stale TextureHandle (generation 7, current 9)`); there is no setting that removes the
  check.
* `[GPU-2]` *(changed in 0.9.15)* `device.begin_frame() -> Option[Frame]`; `None` is normal (a minimised window). A `Frame` has
  one owner and is not `Copy`, and `end_frame(owns frame: Frame)` takes it over (`device.end_frame(frame)`
  ends the name `frame`), so ending a frame that never began, or ending one twice, is refused when
  compiling.
* `[GPU-3]` *(changed in 0.9.15)* `frame.arena()` is the frame's transient memory manager, an instance that lives as long as
  the `Frame`: a value made `using` it (`[ALC-1]`) lives no longer than the frame (`E3063`), so nothing
  made in it outlives `end_frame`.

## D.1 Recording and access states

```ember,fragment
fn draw(ref mut frame: Frame, pipeline: PipelineHandle, vbo: BufferHandle, ibo: BufferHandle, index_count: int):
    with cmd = frame.command_list():
        cmd.begin_render_pass(RenderPassBegin(target=None))    # None: the swapchain
        cmd.bind_pipeline(pipeline)
        cmd.bind_vertex_buffer(0, vbo)
        cmd.bind_index_buffer(ibo)
        cmd.draw_indexed(index_count)                          # checked against `ibo` (`[GPU-12]`)
        cmd.end_render_pass()
```

Each resource has an access state the device tracks: owned by the CPU; recorded in frame N; in
flight in frame N until that frame's fence is observed; retired (destroyed while in flight, freed
later).

* `[GPU-4]` *(changed in 0.9.9)* Writing or mapping a resource that the GPU may still be reading is a
  panic in every profile (`buffer is in flight (frame 42); use a per-frame ring or wait`).
  `Ring[T, N]` holds one resource per frame in flight (`ring.current(frame)`).
* `[GPU-5]` Binding a resource declares its access; `cmd.read(h)`/`cmd.write(h)` declare it
  explicitly; barriers are derived from the declarations; `cmd.barrier(…)` remains available.
  `unsafe: cmd.native()` exposes the backend's command buffer.
* `[GPU-12]` *(new in 0.9.15)* **Every draw and dispatch is checked against what is bound, in every profile.** The
  command list keeps, for each bound vertex, instance and index buffer, how many elements it holds (its
  size over its stride or index width), and before recording a draw compares the ranges the draw reads
  with them: `draw(vertex_count, first_vertex, instance_count, first_instance)` against every bound
  vertex buffer and every instance-rate buffer, `draw_indexed(index_count, first_index, …)` against the
  bound index buffer, and `dispatch(x, y, z)` against the device's group-count limits. One compare per
  range per draw; a range outside stops the program, naming the call, the numbers and the buffer
  (`panic: cmd.draw_indexed(1000000) reads indices 0 to 999999, but the index buffer bound (ibo)
  holds 3`). What these compares cannot see — the vertex numbers stored in an index buffer, an
  indirect draw's arguments in GPU memory, a shader's own reads — is covered by robust buffer access,
  which the device turns on wherever the backend offers it, so such a read past a buffer yields zero
  or nothing instead of other memory. A device whose backend cannot offer it says so
  (`device.robust_access()` is `false`), and its use is an asserted fact that `ember tcb` lists
  (`[TCB-1]`). `unsafe: cmd.native()` records without these compares.
* `[GPU-6]` *(changed in 0.9.15)* `device.destroy(h)` invalidates the handle at once and destroys the object after the GPU
  has finished the frames that may use it. Ending the device waits for the GPU and flushes every
  retirement list.
* `[GPU-7]` *(changed in 0.9.15)* In `debug`, ending a device while handles are still live reports each leaked handle
  with the place it was created.
* `[GPU-8]` `History[T]` owns the per-frame copies of a temporal resource
  (`gpu.History[TextureHandle].new(device, desc, count=2)`) and gives `(prev, cur)` each frame,
  forbidding reads of `cur` and writes of `prev` within a frame.
* `[GPU-9]` *(changed in 0.9.15)* `std.gpu.graph` is an optional render-graph layer over `CommandList`: passes declare reads
  and writes, `graph.compile()` orders them, aliases transient resources with disjoint lifetimes and
  inserts transitions, and `pass.native(fn(ref mut cmd) => …)` records directly inside a graph.

## D.2 Shader interfaces

* `[GPU-10]` `ember shader-bind <reflection.json>`, reading the reflection a SPIR-V toolchain produces,
  generates a module with a `@gpu_layout(std140 | std430)` struct per uniform or storage block, with
  compile-time layout assertions (`E8001` on a mismatch), a typed binding interface, the push-constant
  struct and the vertex-input layout; binding a resource set built for another interface is a type
  error.
* `[GPU-11]` Ember assumes no shader language: anything that produces SPIR-V and reflection works.
* A kernel language (`@gpu fn`) is reserved for a later version; `@gpu` and `std.gpu.kernel` are
  reserved names.
