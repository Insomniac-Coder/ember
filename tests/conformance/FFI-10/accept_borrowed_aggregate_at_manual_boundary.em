#$ test: parse-pass
#$ rules: FFI-10, FFI-9

struct Packet:
    value: i32

unsafe extern "C":
    fn inspect(packet: Packet) -> i32

fn main():
    pass
