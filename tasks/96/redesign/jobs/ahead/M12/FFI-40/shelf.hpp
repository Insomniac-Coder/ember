// The C++ side of the M12 FFI-40 tests: a method's parameters by `T&`, `const T&` and `T&&`, a
// method that changes its object, and one whose `T&` result no contract names.
#pragma once

struct Splash {
    int size;
};

class Cache {
public:
    Cache() noexcept {}
    ~Cache() {}
    void count(const Splash& s) noexcept { total_ += s.size; }
    void grow(Splash& s) noexcept { s.size += 1; }
    void add(Splash&& s) noexcept { total_ += s.size; s.size = 0; }
    int total() const noexcept { return total_; }
    void clear() noexcept { total_ = 0; }

private:
    int total_ = 0;
};

class Node {
public:
    Node() noexcept {}
    Node& parent() noexcept { return *this; }
};
