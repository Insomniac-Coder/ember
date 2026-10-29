#$ test: run-pass
#$ rules: FFI-8, UNS-5
#$ warning[L3015]: `register_write` is a C function the program calls in unsafe code
#$ warning[L3015]: `register_read` is a C function the program calls in unsafe code
#$ warning[L3015]: `device_masked` is a C function the program calls in unsafe code
#$ profiles: debug, development, release
#$ stdout: 7 7
#$ stdout: 12 12 4
# `[FFI-8]`, `[UNS-5]` — C's `volatile T*` is `std.mem.Volatile[*mut T]` (`Volatile[*T]` for
# `const volatile T*`): to C the pointer, and its own reads and writes are volatile
# (`read_volatile`, `write_volatile`, in `unsafe`), each one made where it is written.

from std.mem import Volatile
import c "registers.h"

fn main():
    word: u32 = 0
    reg = Volatile.new(ref_to_ptr(ref mut word))
    first: u32 = 0
    through_c: u32 = 0
    unsafe:  # SAFETY: `word` lives for every access, through C and through `reg`
        c.register_write(reg, 7)
        first = reg.read_volatile()
        through_c = c.register_read(Volatile.new(reg.as_ptr() as *u32))
    println(first, through_c)
    device = c.Device(status = reg, mask = 4)
    status: u32 = 0
    masked: u32 = 0
    unsafe:  # SAFETY: as above
        reg.write_volatile(12)
        status = device.status.read_volatile()
        masked = c.device_masked(device)
    println(word, status, masked)
