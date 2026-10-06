#include <cstdint>
#include <cstdio>
template <class F> static int64_t apply(F f, int64_t v) { return f(v); }
int main() {
    auto step = [](int64_t x) { return x * 3 + 1; };
    int64_t total = 0;
    for (int64_t i = 0; i < 200000000; i++) total += apply(step, i) % 7;
    std::printf("%lld\n", (long long)total);
    return 0;
}
