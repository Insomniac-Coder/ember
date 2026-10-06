#include <stdint.h>
#include <stdio.h>
int main(void) {
    int64_t count = 0;
    for (int64_t py = 0; py < 1000; py++)
        for (int64_t px = 0; px < 1000; px++) {
            double x0 = (double)px * 3.0 / 1000.0 - 2.0;
            double y0 = (double)py * 2.0 / 1000.0 - 1.0;
            double x = 0.0, y = 0.0;
            int64_t i = 0;
            while (i < 200 && x * x + y * y <= 4.0) {
                double xt = x * x - y * y + x0;
                y = 2.0 * x * y + y0;
                x = xt;
                i = i + 1;
            }
            count += i;
        }
    printf("%lld\n", (long long)count);
    return 0;
}
