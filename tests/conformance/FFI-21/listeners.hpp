// The C++ side of accept_a_cpp_callback_with_user_data.em: C++ functions that call back through a
// function pointer and the user data passed beside it.
#pragma once
#include <utility>
#include <vector>

using Visit = void (*)(void* context, int value);

class Bus {
public:
    Bus() noexcept {}
    ~Bus() {}
    void add(int value) noexcept { values_.push_back(value); }
    void each(Visit visit, void* context) const noexcept {
        for (int value : values_) visit(context, value);
    }

private:
    std::vector<int> values_;
};

// Listeners C++ keeps until `unlisten` with the same arguments.
using Listener = int (*)(int event, void* data);
// A template, so the import leaves it to C++ ([FFI-17b]).
template <int Registry = 0> std::vector<std::pair<Listener, void*>>& listeners() {
    static std::vector<std::pair<Listener, void*>> all;
    return all;
}
inline void listen(Listener listener, void* data) noexcept { listeners<>().push_back({listener, data}); }
inline void unlisten(Listener listener, void* data) noexcept {
    auto& all = listeners<>();
    for (auto at = all.begin(); at != all.end(); ++at) {
        if (at->first == listener && at->second == data) {
            all.erase(at);
            return;
        }
    }
}
inline int emit(int event) noexcept {
    int sum = 0;
    for (auto& entry : listeners<>()) sum += entry.first(event, entry.second);
    return sum;
}
inline int listening() noexcept { return static_cast<int>(listeners<>().size()); }

inline int sum_with(int (*map)(void*, int), void* context, int n) noexcept {
    int sum = 0;
    for (int i = 0; i < n; ++i) sum += map(context, i);
    return sum;
}
