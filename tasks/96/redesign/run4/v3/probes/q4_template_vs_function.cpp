#include <chrono>
#include <cstdio>
#include <functional>
#include <vector>

static long long total = 0;

template <typename F>
__attribute__((noinline)) void for_each_template(const std::vector<int>& v, F f) {
    for (int x : v) f(x);
}

__attribute__((noinline)) void for_each_function(const std::vector<int>& v, std::function<void(int)> f) {
    for (int x : v) f(x);
}

int main(int argc, char** argv) {
    std::vector<int> v(100000000);
    for (size_t i = 0; i < v.size(); i++) v[i] = (int)(i % 7);
    bool use_template = argc > 1 && argv[1][0] == 't';
    auto t0 = std::chrono::steady_clock::now();
    if (use_template) for_each_template(v, [](int x) { total += x; });
    else for_each_function(v, [](int x) { total += x; });
    auto t1 = std::chrono::steady_clock::now();
    printf("%s ms=%.1f total=%lld\n", use_template ? "template" : "std::function",
           std::chrono::duration<double, std::milli>(t1 - t0).count(), total);
}
