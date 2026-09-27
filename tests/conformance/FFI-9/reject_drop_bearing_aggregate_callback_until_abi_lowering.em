#$ test: compile-fail
#$ rules: FFI-9
#$ profiles: debug
#$ error[E0900]: `extern "C" fn` pointers with drop-bearing values are not implemented yet

struct Packet:
    x: i32

    fn drop(mut self):
        pass

fn main():
    callback: extern "C" fn(Packet) -> i32
