# Part XXI — RageV Integration Plan

RageV today: a C++ static library (`RageV`) linked into `RageVEditor.exe`/`RageVRuntime.exe`; a backend-agnostic RHI over Vulkan 1.3 (dynamic rendering, synchronization2, `volk`, VMA) and OpenGL 4.5 DSA; `.rvshader` → glslang → SPIR-V (+ SPIRV-Cross for GL and reflection); a sparse-set ECS with `Entity = 20-bit index | 12-bit version`; a render/frame graph with temporal history, RT/GI signal passes; Jolt, miniaudio, ImGui, yaml-cpp, cgltf; C# scripting hosted through `DotNetHost` where **managed → native is a fixed-order table of `__cdecl` function pointers (`NativeApi`) with a protocol version, and native → managed is a set of static blittable entry points**; every crossing value is blittable, entities cross as `uint64_t` UUIDs, strings as UTF-8 copied at the boundary. Editor-visible script fields are declared with `RVShowInEditor` markers read by `rvgen`.

The plan replaces nothing at first and adds Ember beside C#, matching the existing boundary exactly.

## XXI.1 Stage 0 — Ember as a second scripting language (needs Phases 0–5)

**Goal:** an Ember package `ragev_scripts` builds to `ragev_scripts.dll` (`kind = "cdylib"`) that the engine loads next to (or instead of) the managed assembly, using the same `NativeApi` table.

1. **API package.** `ragev_api` (an Ember library) imports `RageV/src/RageV/Managed/Interop.h` via `import c` with an overlay that turns `NativeApi` into a typed table and wraps each entry:

```ember
## ragev_api/src/lib.em
import c "RageV/src/RageV/Managed/Interop.h" with (project="ragev", overlay="overlays/interop.em")

@derive(Copy, Eq, Hash, Debug)
struct Entity:                                         # UUID across the boundary; never a pointer (play-mode restore)
    id: u64
    const NULL: Entity = Entity(0)

static API: Atomic[*const c.NativeApi] = Atomic(null())   # set once by ember_module_init; read relaxed

fn api() -> ref c.NativeApi:
    unsafe: return API.load(Relaxed).deref()             # SAFETY: set before any script runs; table lives for the process

pub fn log(level: LogLevel, msg: str):
    with c = msg.to_cstring():                           # copied at the boundary, as the C# side does
        unsafe: (api().Log.unwrap())(level as i32, c.as_ptr())

pub fn position(e: Entity) -> Option[Vec3]:
    out: Vec3
    ok = unsafe: (api().GetPosition.unwrap())(e.id, ref mut out as *mut c.Vector3)   # Vec3 is layout-identical to Vector3
    return Some(out) if ok != 0 else None
```

   `ember_module_init(const ember_host_api* host)` receives a struct `{ uint32_t protocol; const NativeApi* api; }`; `[RV-1]` the module refuses to load (`return -1` + log) if `protocol != RAGEV_PROTOCOL_VERSION` baked at build time — the same guard `Interop.ProtocolVersion` provides for C#.

2. **Script model.** A script is a class deriving from `ragev.Script` with the same lifecycle names the C# `ScriptHost` exposes (`on_create`, `on_update(dt)`, `on_fixed_update(dt)`, `on_destroy`, `on_collision(e: CollisionEvent)`), registered by `@derive(Script)`:

```ember
@derive(Script)
class Swing(Script):
    @field(range=(0.0, 2.0))           # editor-visible, serialised — the `RVShowInEditor` equivalent
    amplitude: f32 = 0.34
    @field
    target: Entity = Entity.NULL
    t: f32 = 0.0                       # not a field: runtime-only

    override fn on_update(mut self, dt: f32):
        self.t += dt
        if Some(p) = position(self.entity):
            set_position(self.entity, p + Vec3(0, sin(self.t) * self.amplitude, 0))

    @callable                          # nameable by a UI button, the `RVCallable` equivalent
    fn ring(mut self): log(Info, "ring")
```

   `@derive(Script)` generates: a comptime registry entry `{name, create: extern "C" fn(u64) -> *void, field descriptors (name, type tag, offset, attrs), callables}`, and the class is `Sync`-unconstrained (scripts are main-thread only; the module declares `threads = main`).

3. **Exported table.** The module exports one table (RageV's own style, "order is the ABI"):

```ember
@export_table("RvEmberScriptApi", protocol = 1)
pub struct ScriptModuleApi:
    script_count:        extern "C" fn() -> u32
    script_name:         extern "C" fn(u32) -> cstr
    script_field_count:  extern "C" fn(u32) -> u32
    script_field_desc:   extern "C" fn(u32, u32, *mut FieldDesc) -> i32
    instance_create:     extern "C" fn(u32 /*script*/, u64 /*entity*/) -> u64 /*instance id*/
    instance_destroy:    extern "C" fn(u64) -> void
    instance_set_field:  extern "C" fn(u64, u32, *const u8, u32) -> i32     # raw bytes from the scene file, per type tag
    instance_get_field:  extern "C" fn(u64, u32, *mut u8, u32) -> i32
    instance_call:       extern "C" fn(u64, cstr) -> i32
    on_create/on_update/on_fixed_update/on_destroy/on_collision: extern "C" fn(u64, ...) -> i32
    reload_begin:        extern "C" fn(*mut u8, u32) -> u32      # serialise all instance fields (hot reload)
    reload_end:          extern "C" fn(*const u8, u32) -> i32    # restore after a new DLL is loaded
```

   Instances are addressed by a `u64` from a `Pool[Box[dyn Script]]` (generational; never a pointer), so a stale instance id fails safely. `[RV-2]` No Ember object pointer ever crosses the boundary.

4. **Engine side** (C++, small): `EmberHost.cpp` next to `DotNetHost.cpp`: `LoadLibrary`, resolve `ember_module_init` and `RvEmberScriptApi`, register script types into `ScriptRegistry` exactly as managed scripts are registered, forward lifecycle calls, and drive hot reload (`reload_begin` → unload → rebuild → load → `reload_end`). Ember-side `ember_rt_init` uses `cfg.log = RageV::Log`, `cfg.alloc` left default (mimalloc) or routed to the engine allocator.

5. **Build.** `RageVScripts/CMakeLists.txt` uses `ember_add_library(ragev_scripts KIND cdylib SOURCES …)` with `--cc-flags-from-target RageV`; it is optional like the .NET SDK (`find_program(EMBER ember)` → skip with a status message).

**Exit criteria:** the SampleProject's C# sample scripts ported line-for-line to Ember; play mode, stop/restore, editor field editing, hot reload all work with either language; per-call overhead of `position()` measured ≤ the C# function-pointer path.

## XXI.2 Stage 1 — Engine C API and idiomatic bindings (Phase 5)

Grow `Interop.h` (or a new `RageVCApi.h` generated by extending `rvgen`) into a C surface for: scene queries, component get/set (blittable components), input, audio, physics queries (Jolt via the engine), asset handles, debug draw, and RHI-independent renderer settings. Every entry gets an overlay contract; `ragev_api` exposes idiomatic Ember (`Result`, `Option`, `Span`) over it. `[RV-3]` Component types shared with the engine are `@layout(c)` Ember structs mirrored from the C++ headers with comptime layout assertions against the `.embind` layouts (`[FFI-5]`).

## XXI.3 Stage 2 — Systems in Ember over the engine ECS (Phase 6)

Two options, decided by measurement:

* **(a) Bridge:** the engine exposes `ECS_Pool_Data(type_hash) -> {entities*, count, components*, stride}`; Ember wraps it as `Query` over borrowed `Span`s (zero-copy). Systems written in Ember run inside `Scene::Update` between engine systems. Requires `[ECS-2]`'s identical `TypeHash`.
* **(b) Ember-owned ECS:** `std.ecs.World` becomes the store; the engine's C++ systems read through the same C API in reverse. Larger change; only if (a) shows unacceptable overhead.

Target systems for Ember first: particle simulation (`SoA`, `@parallel`, `@simd`), CPU frustum/occlusion pre-pass feeding `GpuCull`, animation pose evaluation, scene graph transform walk (`Get<T>` two-loads invariant preserved via `[ECS-2]`).

## XXI.4 Stage 3 — Renderer orchestration (Phase 7)

Bind `RHIDevice`, `RHICommandList`, `RHIResourceSet`, `RHIPipeline` through the C++ importer (`import cpp … classes=[…]`) with an overlay that maps the engine's raw pointers to `std.gpu` handles (`Handle[Texture]` ↔ `RHITexture*` via a `Pool` on the Ember side; the engine's own deferred destruction stays authoritative). Then move **frame orchestration** — `FrameGraphBuilder`-level decisions: which passes run, at what resolution, with which history, and the `TemporalHistory` bookkeeping — into Ember, using `std.gpu.History` and access-state checks to make "GI signal read by nobody" and "history refused by validity flag" (the traps recorded in `HANDOFF.md`) into compile-time or debug-time errors: a `History` whose `cur` is never bound in a frame is reported by the device in debug (`W-runtime: history 'gi' written but not read this frame`). Shader interfaces come from `ember shader-bind` on the existing `shaderinfo` reflection so that binding 16's dual meaning (`RayRates.w` bit 24) becomes a typed enum in the interface rather than a comment.

`[RV-4]` The Vulkan and OpenGL backends, `volk`, VMA, swapchain and window code remain C++ ("native islands") indefinitely; nothing in Ember requires their rewrite. Vendor extensions remain reachable through `import c "vulkan/vulkan.h"` + `unsafe: cmd.native()`.

## XXI.5 Stage 4 — RT/GI signal scheduling and tools (v1.1+)

Once Stage 3 is stable: the signal-processing chains (trace → guidance downsample → contract → upsample) are pass sequences whose resolution, ordering and buffer aliasing are orchestration — the kind of code where Ember's `@noalloc` frame arenas, `History`, and the graph's automatic transient lifetimes pay off. Offline tools (`diff_still`, `terrain_lod` replicas currently in Python) become `ember` programs sharing the engine's math and serialisation types.

## XXI.6 Evaluation matrix (measure before moving each area)

| Area | Stage | Ember mechanism | Pass criterion |
|---|---|---|---|
| Gameplay scripts | 0 | classes, `@derive(Script)`, hot reload | feature parity with C#; ≤ C# per-call cost |
| Editor fields/callables | 0 | `@reflect` + `@field` | inspector parity |
| Particles / SoA sims | 2 | `SoA`, `@parallel`, `@simd` | ≤ 1.10× C++ |
| ECS systems | 2 | `Query`, access sets | `Get<T>` two loads; iteration order stable |
| Transform walk | 2 | `Query`, `ref mut` | ≤ 1.05× C++ |
| Frame orchestration | 3 | `std.gpu`, `History`, `Frame` | zero validation errors; histories statically paired |
| Shader interface | 3 | `ember shader-bind` | mismatched binding = compile error |
| Vulkan/GL backends | never | native island | n/a |
| Platform/window | never | native island | n/a |

---

