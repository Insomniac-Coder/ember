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
    fn begin_frame(mut self) -> Option[Frame]           # None: skip this frame; no end_frame
    fn end_frame(mut self, owned frame: Frame)
    fn create_texture(mut self, desc: TextureDesc) -> Result[TextureHandle, GpuError]
    fn create_buffer(mut self, desc: BufferDesc) -> Result[BufferHandle, GpuError]
    fn destroy(mut self, h: TextureHandle)                 # physical destruction is deferred
    fn frames_in_flight(self) -> int
```

* `[GPU-1]` *(changed in 0.9.9)* GPU objects are named by generational handles (`[HND-1]`), which are
  `Copy` and `Send`. A stale handle is detected by its generation **in every profile**
  (`panic: stale TextureHandle (generation 7, current 9)`); there is no setting that removes the
  check.
* `[GPU-2]` `device.begin_frame() -> Option[Frame]`; `None` is normal (a minimised window). `Frame` is
  move-only and `end_frame(frame)` consumes it, so ending a frame that never began is a type error.
* `[GPU-3]` `frame.arena()` is the frame's transient allocator; its region is the `Frame`, so nothing
  allocated from it outlives `end_frame`.

## D.1 Recording and access states

```ember,fragment
fn draw(mut frame: Frame, pipeline: PipelineHandle, vbo: BufferHandle, index_count: int):
    with cmd = frame.command_list():
        cmd.begin_render_pass(RenderPassBegin(target=None))    # None: the swapchain
        cmd.bind_pipeline(pipeline)
        cmd.bind_vertex_buffer(0, vbo)
        cmd.draw_indexed(index_count)
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
* `[GPU-6]` `device.destroy(h)` invalidates the handle at once and destroys the object after the GPU
  has finished the frames that may use it. Dropping the device waits for the GPU and flushes every
  retirement list.
* `[GPU-7]` In `debug`, dropping a device while handles are still live reports each leaked handle
  with the place it was created.
* `[GPU-8]` `History[T]` owns the per-frame copies of a temporal resource
  (`gpu.History[TextureHandle].new(device, desc, count=2)`) and gives `(prev, cur)` each frame,
  forbidding reads of `cur` and writes of `prev` within a frame.
* `[GPU-9]` `std.gpu.graph` is an optional render-graph layer over `CommandList`: passes declare reads
  and writes, `graph.compile()` orders them, aliases transient resources with disjoint lifetimes and
  inserts transitions, and `pass.native(fn(cmd) => …)` records directly inside a graph.

## D.2 Shader interfaces

* `[GPU-10]` `ember shader-bind <reflection.json>`, reading the reflection a SPIR-V toolchain produces,
  generates a module with a `@gpu_layout(std140 | std430)` struct per uniform or storage block, with
  compile-time layout assertions (`E8001` on a mismatch), a typed binding interface, the push-constant
  struct and the vertex-input layout; binding a resource set built for another interface is a type
  error.
* `[GPU-11]` Ember assumes no shader language: anything that produces SPIR-V and reflection works.
* A kernel language (`@gpu fn`) is reserved for a later version; `@gpu` and `std.gpu.kernel` are
  reserved names.
