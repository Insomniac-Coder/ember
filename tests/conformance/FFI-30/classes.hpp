// The C++ side of the FFI-30 tests of one C++ header imported by two modules (D-879).
#pragma once

class Counter {
public:
    Counter() noexcept : count_(0) {}
    ~Counter() {}
    void add(int n) noexcept { count_ += n; }
    int count() const noexcept { return count_; }

private:
    int count_;
};

inline int doubled(int n) noexcept { return n * 2; }
