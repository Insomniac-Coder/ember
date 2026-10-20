// The C++ side of the FFI-40 tests: methods, and parameters by reference and by rvalue reference.
#pragma once

struct Item {
    int value;
};

class Bag {
public:
    Bag() noexcept {}
    ~Bag() {}
    void put(Item&& item) noexcept { total_ += item.value; }
    void add(const Item& item) noexcept { total_ += item.value; }
    void merge(const Bag& other) noexcept { total_ += other.total_; }
    void drain(Bag& other) noexcept { total_ += other.total_; other.total_ = 0; }
    int total() const noexcept { return total_; }
    int& slot() noexcept { return total_; }
    const int& peek() const noexcept { return total_; }
    int& checked() { return total_; }
    void take() && noexcept {}

private:
    int total_ = 0;
};

inline void reset(int& value) noexcept { value = 0; }
inline int both(const Bag& a, const Bag& b) noexcept { return a.total() + b.total(); }
