#$ test: compile-fail
#$ rules: FFI-10, FFI-9
#$ profiles: debug
#$ error[E0900]: a borrowed aggregate at a C boundary needs by-value ABI lowering, which is not implemented yet

struct Packet:
    value: i32

unsafe extern "C":
    fn inspect(packet: Packet) -> i32

fn main():
    pass
