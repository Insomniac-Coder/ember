// The C++ side of the M12 FFI-42a test: a container whose iterators are pointers, with a method that
// only reads it, one that changes it, and a `const` one that changes a `mutable` member.
#pragma once

class Row {
public:
    Row() noexcept {}
    int* begin() noexcept { return cells; }
    int* end() noexcept { return cells + 3; }
    int first() const noexcept { return cells[0]; }
    void bump() noexcept { cells[0] += 1; }
    int looked() const noexcept { return ++looks; }

private:
    int cells[3] = {5, 6, 7};
    mutable int looks = 0;
};
