// The C++ side of the FFI-42 and FFI-42a iteration tests: a container whose iterators are a
// standard library's class, and one whose iterators are pointers.
#pragma once
#include <vector>

class Bag {
public:
    Bag() {}
    void put(int value) { items.push_back(value); }
    int size() const noexcept { return static_cast<int>(items.size()); }
    std::vector<int>::iterator begin() noexcept { return items.begin(); }
    std::vector<int>::iterator end() noexcept { return items.end(); }
private:
    std::vector<int> items;
};

struct Row {
    int cells[3];
    int* begin() noexcept { return cells; }
    int* end() noexcept { return cells + 3; }
};
