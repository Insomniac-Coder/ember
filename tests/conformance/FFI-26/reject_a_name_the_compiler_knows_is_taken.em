#$ test: compile-fail
#$ rules: FFI-26, RT-5, FFI-28
# `[FFI-26]` — a name the compiler knows is taken is refused even in the unsafe form (`E0104`): a
# function of the C standard library, a name beginning with the runtime's prefix (`[RT-5]`), and the
# module-protocol names of `[FFI-28]`, a table's `Name_vN` included. (`main` and a symbol two functions define: the other tests.)

# SAFETY: test
@unsafe(export("realloc"))    #$ error[E0104]: `realloc` is a function of the C standard library, and exporting `same_block` under its name would replace it for the whole process, the runtime's own allocations included; choose another name
fn same_block(n: i32) -> i32:
    return n

# SAFETY: test
@unsafe(export("sqrtf"))    #$ error[E0104]: `sqrtf` is a function of the C standard library, and exporting `root` under its name would replace it for the whole process; choose another name
fn root(x: f32) -> f32:
    return x

# SAFETY: test
@unsafe(export())    #$ error[E0104]: `printf` is a function of the C standard library
fn printf() -> i32:
    return 0

# SAFETY: test
@unsafe(export("ember_rt_alloc"))    #$ error[E0104]: `ember_rt_alloc` begins with `ember_`, the runtime's prefix
fn alloc_like() -> i32:
    return 1

# SAFETY: test
@unsafe(export("ember_module_init"))    #$ error[E0104]: `ember_module_init` is a name of the module protocol
fn start() -> i32:
    return 2

# SAFETY: test
@unsafe(export("ember_module_protocol_v3"))    #$ error[E0104]: `ember_module_protocol_v3` is a name of the module protocol
fn marker() -> i32:
    return 3

pub extern "C" fn on_tick() -> i32:
    return 4

# SAFETY: test
@unsafe(export_table("ember_api", protocol=1))    #$ error[E0104]: `ember_api_v1` begins with `ember_`, the runtime's prefix
pub struct Api:
    on_tick: extern "C" fn() -> i32

fn main():
    pass
