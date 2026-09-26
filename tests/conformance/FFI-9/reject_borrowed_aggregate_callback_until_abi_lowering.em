#$ test: compile-fail
#$ rules: FFI-9
#$ profiles: debug
#$ error[E0900]: `extern "C" fn` pointers with drop-bearing values or borrowed aggregates are not implemented yet

struct Packet:
    x: i32
    y: i32

fn main():
    callback: extern "C" fn(Packet) -> i32
