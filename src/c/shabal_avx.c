#include "shabal_avx.h"
#include <immintrin.h>
#include <string.h>
#include "common.h"
#include "mshabal_128_avx.h"
#include "sph_shabal.h"

mshabal128_context global_128;
mshabal128_context_fast global_128_fast;

void init_shabal_avx() {
    mshabal_init_avx(&global_128, 256);
    global_128_fast.out_size = global_128.out_size;
    for (uint64_t i = 0; i < 176; i++) global_128_fast.state[i] = global_128.state[i];
    global_128_fast.Whigh = global_128.Whigh;
    global_128_fast.Wlow = global_128.Wlow;
}

void find_best_deadline_avx(char *scoops, uint64_t nonce_count, char *gensig,
                            uint64_t *best_deadline, uint64_t *best_offset) {
    uint64_t d0 = 0, d1 = 0, d2 = 0, d3 = 0;
    char term[32];
    write_term(term);

    // local copy of global fast context
    mshabal128_context_fast x;
    memcpy(&x, &global_128_fast, sizeof(global_128_fast));

    // prepare shabal inputs
    union {
        mshabal_u32 words[16 * MSHABAL128_VECTOR_SIZE];
        __m128i data[16];
    } u1, u2;

    for (uint64_t i = 0; i < 16 * MSHABAL128_VECTOR_SIZE / 2; i += MSHABAL128_VECTOR_SIZE) {
        size_t o = i;
        u1.words[i + 0] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 1] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 2] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 3] = *(mshabal_u32 *)(gensig + o);
        u2.words[i + 0 + 32] = *(mshabal_u32 *)(term + o);
        u2.words[i + 1 + 32] = *(mshabal_u32 *)(term + o);
        u2.words[i + 2 + 32] = *(mshabal_u32 *)(term + o);
        u2.words[i + 3 + 32] = *(mshabal_u32 *)(term + o);
    }

    for (uint64_t i = 0; i < nonce_count;) {
        if (i + 4 <= nonce_count) {
            // load and align data for SIMD
            for (uint64_t j = 0; j < 16 * MSHABAL128_VECTOR_SIZE / 2; j += MSHABAL128_VECTOR_SIZE) {
                size_t o = j;
                u1.words[j + 0 + 32] = *(mshabal_u32 *)(&scoops[(i + 0) * 64] + o);
                u1.words[j + 1 + 32] = *(mshabal_u32 *)(&scoops[(i + 1) * 64] + o);
                u1.words[j + 2 + 32] = *(mshabal_u32 *)(&scoops[(i + 2) * 64] + o);
                u1.words[j + 3 + 32] = *(mshabal_u32 *)(&scoops[(i + 3) * 64] + o);
                u2.words[j + 0] = *(mshabal_u32 *)(&scoops[(i + 0) * 64 + 32] + o);
                u2.words[j + 1] = *(mshabal_u32 *)(&scoops[(i + 1) * 64 + 32] + o);
                u2.words[j + 2] = *(mshabal_u32 *)(&scoops[(i + 2) * 64 + 32] + o);
                u2.words[j + 3] = *(mshabal_u32 *)(&scoops[(i + 3) * 64 + 32] + o);
            }

            mshabal_deadline_fast_avx(&x, &u1, &u2, &d0, &d1, &d2, &d3);

            SET_BEST_DEADLINE(d0, i + 0);
            SET_BEST_DEADLINE(d1, i + 1);
            SET_BEST_DEADLINE(d2, i + 2);
            SET_BEST_DEADLINE(d3, i + 3);
            i += 4;
        } else {
            sph_shabal_deadline_fast(&scoops[i * 64], gensig, &d0);
            SET_BEST_DEADLINE(d0, i);
            i++;
        }
    }
}
