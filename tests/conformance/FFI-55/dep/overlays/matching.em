# Another package's overlay of `lib.h`, written for the header these tests import: its hash is recorded.

unsafe overlay c "lib.h" with (header_hash = "blake3:98b885ab509b9a92f226842de172d3894e4a701b75b79bab65743c8706a06531"):  # SAFETY(ffi): each entry matches its declaration in the header
    fn lib_twice(x)
