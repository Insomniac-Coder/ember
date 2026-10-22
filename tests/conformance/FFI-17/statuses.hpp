// The C++ side of the FFI-17 test of `out` and `status` words on functions Ember reaches through
// a wrapper (D-808): a method writing through pointers, and a status a function returns.
#pragma once

enum class Status : int { ok = 0, odd = 1 };

class Splitter {
public:
    Splitter() noexcept {}
    // Halves `value` into `half`: `Status::odd` for an odd value.
    Status split(int value, int* half) const noexcept {
        if (value % 2 != 0) return Status::odd;
        *half = value / 2;
        return Status::ok;
    }
    // Writes the bounds of `value`'s decade through two pointers, and gives the decade.
    int decade(int value, int* low, int* high) const {
        *low = value / 10 * 10;
        *high = *low + 9;
        return value / 10;
    }
};
