# Part XVII — GPU Host Model and Shader Interface

Ember does not compile shaders in v1. It specifies the host-side facilities that RageV's renderer needs, as a library (`std.gpu`) over an abstract device interface that the engine implements in C++ (its existing RHI) and exposes through the FFI, or that a future Ember-native backend implements directly. Nothing here is engine-specific; everything maps one-to-one onto RageV's RHI shape (`BeginFrame → record → EndFrame`, per-frame-in-flight resource sets, deferred destruction).

## XVII.1 Handles

```ember
type TextureHandle  = Handle[gpu.Texture]
type BufferHandle   = Handle[gpu.Buffer]
type PipelineHandle = Handle[gpu.Pipeline]
type ResourceSetHandle = Handle[gpu.ResourceSet]
```

`[GPU-1]` All GPU objects are referred to by generational handles (Part IX §6); physical objects live in the device's pools. Handles are `Copy`, `Send`, and safe to store anywhere; a stale handle is detected by generation (`E-panic: stale TextureHandle (gen 7, current 9)` in debug/release, unchecked in shipping only with `gpu.validate = false`).

## XVII.2 Device and frame lifecycle

```ember
interface Device:                                     # implemented by the RHI bridge
    fn begin_frame(mut self) -> Option[Frame]         # None ⇒ skip this frame (resized/minimised); do NOT call end_frame
    fn end_frame(mut self, owned frame: Frame)
    fn create_texture(mut self, desc: TextureDesc) -> Result[TextureHandle, GpuError]
    fn create_buffer(mut self, desc: BufferDesc) -> Result[BufferHandle, GpuError]
    fn destroy(mut self, h: TextureHandle)            # logical destruction; physical destruction is deferred (§4)
    fn frames_in_flight(self) -> u32
    fn caps(self) -> DeviceCaps
    ...

struct Frame:                                         # move-only; @view over the device
    index: u32                                        # frame-in-flight slot
    fn command_list(mut self) -> ref mut CommandList
    fn arena(self) -> ref Arena                       # the frame arena (reset at end_frame)
```

* `[GPU-2]` `begin_frame` returning `None` is normal; because `Frame` is move-only and `end_frame` consumes it, the RageV trap "calling EndFrame after a null BeginFrame" is a type error rather than a runtime bug.
* `[GPU-3]` `Frame.arena()` is the per-frame transient allocator; its region is the `Frame`, so nothing allocated from it can survive `end_frame`.

## XVII.3 Command recording and resource access states

```ember
with cmd = frame.command_list():
    cmd.begin_render_pass(RenderPassBegin(target=None))   # None ⇒ swapchain
    cmd.bind_pipeline(pipeline)
    cmd.bind_resource_set(0, set)
    cmd.bind_vertex_buffer(0, vbo)
    cmd.bind_index_buffer(ibo, IndexType.U32)
    cmd.draw_indexed(index_count)
    cmd.end_render_pass()
```

Each resource carries a runtime **access state** tracked by the device:

```
CpuOwned ──(cmd.read/write, bind)──► Recording(frame N) ──(end_frame)──► InFlight(frame N)
   ▲                                                                         │
   └──────────────────(frame N's fence signalled at begin_frame(N + frames_in_flight))──┘
Retired: logically destroyed while InFlight ⇒ physical destruction deferred until CpuOwned
```

* `[GPU-4]` Uploading to or mapping a resource (`device.write_buffer(h, data)`, `device.map(h)`) while it is `InFlight` is an error `E-panic: buffer is in flight (frame 42); use a per-frame ring or wait` in debug/release. The engine's existing per-frame-in-flight duplication (`RHIResourceSet` holds one set per frame) is expressed as `Ring[T, N]` in `std.gpu` (`ring.current(frame)`).
* `[GPU-5]` `cmd.read(h)`/`cmd.write(h)` are optional explicit declarations that let the device derive barriers/transitions (declarative state tracking); binding a resource implies the appropriate declaration. Explicit `cmd.barrier(...)` remains available, and `unsafe: cmd.native()` returns the raw backend command buffer for vendor extensions (`VkCommandBuffer`), i.e. Level 1 of the three-level model.

## XVII.4 Deferred destruction

`[GPU-6]` `device.destroy(h)` invalidates the handle immediately (generation bump) and queues the physical object on the retirement list of the current frame; the runtime destroys it when that frame's fence has been observed signalled. This is RageV's `VulkanDevice::DeferDestruction` made a standard primitive. Dropping the device flushes all retirement lists after a full wait-idle; `[GPU-7]` device drop while handles are still live logs each leaked handle with its creation site in debug.

## XVII.5 Temporal history

```ember
history = gpu.History[TextureHandle].new(mut device, desc, count=2)
prev, cur = history.pair(frame)         # prev: read this frame, cur: write this frame; swapped at end_frame
```

`[GPU-8]` `History[T]` owns `count` handles, exposes `(prev, cur)` per frame, and forbids `cur` being read or `prev` being written in the same frame (checked through the access-state machine). It is registered with the device so its resources are `InFlight`-tracked like any other.

## XVII.6 Render graph (optional layer)

`std.gpu.graph` provides `Graph`, `Pass`, `graph.transient_texture(desc)`, `pass.read(h)`, `pass.write(h)`, `pass.native(fn(cmd) => ...)` and `graph.compile()` which derives first/last use per transient, aliases transients with disjoint lifetimes into shared physical allocations, orders passes by dependencies, and inserts transitions. `[GPU-9]` The graph is a library over `CommandList`; direct recording and native passes remain available inside a graph. Its implementation is Phase 7 and designed to be replaced by RageV's own `RenderGraph`/`FrameGraphBuilder` through the FFI first.

## XVII.7 Shader interface generation

```
foo.rvshader ─glslang─► foo.spv ─(SPIRV-Cross reflection, as RageV's shaderinfo already does)─► foo.reflect.json
                                                                                                    │
                                                        ember shader-bind foo.reflect.json ─► foo_shader.em
```

`[GPU-10]` The generated module contains: a `@gpu_layout(std140|std430)` struct per uniform/storage block with **comptime layout assertions**, a `ShaderInterface` struct with typed binding slots (`set`, `binding`, kind, array size), push-constant struct, and vertex-input layout constants. Binding a `ResourceSet` built from a mismatched interface is a type error, not a validation-layer message. Shader language remains GLSL (or anything that yields SPIR-V); `[GPU-11]` Ember never assumes a shader language.

## XVII.8 Kernel language (v3, reserved)

`@gpu fn` kernels compiling to SPIR-V are out of scope for v1/v2. The keyword `@gpu` is reserved; `std.gpu.kernel` is reserved as a module name.

---
