#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <vector>
int main() {
    std::vector<int64_t> xs;
    for (int64_t i = 0; i < 5000000; i++) xs.push_back((i * 7919) % 1000003);
    std::sort(xs.begin(), xs.end());
    std::printf("%lld %lld %lld\n", (long long)xs[0], (long long)xs[2500000], (long long)xs[4999999]);
    return 0;
}
