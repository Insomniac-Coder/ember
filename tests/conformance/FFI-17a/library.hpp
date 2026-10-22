// The C++ side of the FFI-17a and FFI-17e tests: functions of standard-library types.
#pragma once
#include <memory>
#include <string>
#include <vector>

inline std::vector<int> squares(int n) {
    std::vector<int> out;
    for (int i = 0; i < n; ++i) out.push_back(i * i);
    return out;
}
inline int total(const std::vector<int>& values) noexcept {
    int sum = 0;
    for (int value : values) sum += value;
    return sum;
}
inline void double_all(std::vector<int>& values) noexcept {
    for (int& value : values) value *= 2;
}
inline std::string label(int n) { return "item " + std::to_string(n); }
inline std::string broken() { return std::string("caf\xC3", 4); }

inline int destroyed_counters = 0;
class Counter {
public:
    explicit Counter(int start) noexcept : value_(start) {}
    ~Counter() { ++destroyed_counters; }
    int value() const noexcept { return value_; }

private:
    int value_;
};

inline int destroyed() noexcept { return destroyed_counters; }
inline std::shared_ptr<Counter> shared_counter(int start) { return std::make_shared<Counter>(start); }
inline long owners(const std::shared_ptr<Counter>& counter) noexcept { return counter.use_count(); }
inline std::unique_ptr<Counter> unique_counter(int start) { return std::make_unique<Counter>(start); }
inline std::weak_ptr<Counter> watch(const std::shared_ptr<Counter>& counter) noexcept { return counter; }
