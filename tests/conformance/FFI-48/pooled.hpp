// The C++ side of reject_an_allocator_parameterised_container.em.
#pragma once
#include <cstddef>
#include <new>
#include <vector>

template <typename T> struct Pool {
    using value_type = T;
    Pool() = default;
    template <typename U> Pool(const Pool<U>&) {}
    T* allocate(std::size_t n) { return static_cast<T*>(::operator new(n * sizeof(T))); }
    void deallocate(T* p, std::size_t) { ::operator delete(p); }
};
template <typename T, typename U> bool operator==(const Pool<T>&, const Pool<U>&) { return true; }
template <typename T, typename U> bool operator!=(const Pool<T>&, const Pool<U>&) { return false; }

inline std::vector<int, Pool<int>> pooled(int n) { return std::vector<int, Pool<int>>(n, 1); }
inline int pooled_total(const std::vector<int, Pool<int>>& values) {
    int sum = 0;
    for (int value : values) sum += value;
    return sum;
}
inline std::vector<int> plain(int n) { return std::vector<int>(n, 1); }
