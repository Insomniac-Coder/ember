# Proposed syntax (inferred contracts, answers report 8.4): "When a C function takes a function pointer
# and a void*, and the function pointer's own parameters include a void*, the importer infers
# callback(retained, user_data=<that parameter>)". This C header has that shape, but `buf` is the buffer
# C reads into, and the callback receives it back; it is not user data:
#
#     void async_read(int fd, void *buf, size_t n, void (*done)(void *buf));
#
# The importer takes `buf` out of the Ember signature and passes the trampoline's handle in its place.
# C then writes up to n bytes at the address `slot | generation << 32`: memory corruption, with no
# `unsafe` written anywhere. None of the trampoline's checks runs: the damage is done before any callback.
import c "aio.h" as aio

fn main():
    fd = aio.open_log()
    aio.async_read(fd, 4096, fn(): println("done"))     # the signature the inference produced
