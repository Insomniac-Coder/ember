# Another package's overlay of `lib.h`, written for another version of the header.

unsafe overlay c "lib.h" with (header_hash = "blake3:0000000000000000000000000000000000000000000000000000000000000000"):  # SAFETY(ffi): each entry matches its declaration in the header
    fn lib_twice(x)
