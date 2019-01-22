#include "shabal_avx2.h"
#include <immintrin.h>
#include <string.h>
#include "common.h"
#include "mshabal_256_avx2.h"
#include "sph_shabal.h"

mshabal256_context global_256;
mshabal256_context_fast global_256_fast;

void init_shabal_avx2() {
    mshabal_init_avx2(&global_256, 256);
    global_256_fast.out_size = global_256.out_size;
    for (uint64_t i = 0; i < 352; i++) global_256_fast.state[i] = global_256.state[i];
    global_256_fast.Whigh = global_256.Whigh;
    global_256_fast.Wlow = global_256.Wlow;
}

void find_best_deadline_avx2(char *scoops, uint64_t nonce_count, char *gensig,
                             uint64_t *best_deadline, uint64_t *best_offset) {
    uint64_t d0 = 0, d1 = 0, d2 = 0, d3 = 0, d4 = 0, d5 = 0, d6 = 0, d7 = 0;
    char term[32];
    write_term(term);

    // local copy of global fast context
    mshabal256_context_fast x;
    memcpy(&x, &global_256_fast, sizeof(global_256_fast));

    // prepare shabal inputs
    union {
        mshabal_u32 words[16 * MSHABAL256_VECTOR_SIZE];
        __m256i data[16];
    } u1, u2;

    for (uint64_t i = 0; i < 16 * MSHABAL256_VECTOR_SIZE / 2; i += MSHABAL256_VECTOR_SIZE) {
        size_t o = i / 2;
        u1.words[i + 0] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 1] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 2] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 3] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 4] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 5] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 6] = *(mshabal_u32 *)(gensig + o);
        u1.words[i + 7] = *(mshabal_u32 *)(gensig + o);
        u2.words[i + 0 + 64] = *(mshabal_u32 *)(term + o);
        u2.words[i + 1 + 64] = *(mshabal_u32 *)(term + o);
        u2.words[i + 2 + 64] = *(mshabal_u32 *)(term + o);
        u2.words[i + 3 + 64] = *(mshabal_u32 *)(term + o);
        u2.words[i + 4 + 64] = *(mshabal_u32 *)(term + o);
        u2.words[i + 5 + 64] = *(mshabal_u32 *)(term + o);
        u2.words[i + 6 + 64] = *(mshabal_u32 *)(term + o);
        u2.words[i + 7 + 64] = *(mshabal_u32 *)(term + o);
    }

    for (uint64_t i = 0; i < nonce_count;) {
        if (i + 8 <= nonce_count) {
            // load and align data for SIMD
            for (uint64_t j = 0; j < 16 * MSHABAL256_VECTOR_SIZE / 2; j += MSHABAL256_VECTOR_SIZE) {
                size_t o = j / 2;
                u1.words[j + 0 + 64] = *(mshabal_u32 *)(&scoops[(i + 0) * 64] + o);
                u1.words[j + 1 + 64] = *(mshabal_u32 *)(&scoops[(i + 1) * 64] + o);
                u1.words[j + 2 + 64] = *(mshabal_u32 *)(&scoops[(i + 2) * 64] + o);
                u1.words[j + 3 + 64] = *(mshabal_u32 *)(&scoops[(i + 3) * 64] + o);
                u1.words[j + 4 + 64] = *(mshabal_u32 *)(&scoops[(i + 4) * 64] + o);
                u1.words[j + 5 + 64] = *(mshabal_u32 *)(&scoops[(i + 5) * 64] + o);
                u1.words[j + 6 + 64] = *(mshabal_u32 *)(&scoops[(i + 6) * 64] + o);
                u1.words[j + 7 + 64] = *(mshabal_u32 *)(&scoops[(i + 7) * 64] + o);
                u2.words[j + 0] = *(mshabal_u32 *)(&scoops[(i + 0) * 64 + 32] + o);
                u2.words[j + 1] = *(mshabal_u32 *)(&scoops[(i + 1) * 64 + 32] + o);
                u2.words[j + 2] = *(mshabal_u32 *)(&scoops[(i + 2) * 64 + 32] + o);
                u2.words[j + 3] = *(mshabal_u32 *)(&scoops[(i + 3) * 64 + 32] + o);
                u2.words[j + 4] = *(mshabal_u32 *)(&scoops[(i + 4) * 64 + 32] + o);
                u2.words[j + 5] = *(mshabal_u32 *)(&scoops[(i + 5) * 64 + 32] + o);
                u2.words[j + 6] = *(mshabal_u32 *)(&scoops[(i + 6) * 64 + 32] + o);
                u2.words[j + 7] = *(mshabal_u32 *)(&scoops[(i + 7) * 64 + 32] + o);
            }

            mshabal_deadline_fast_avx2(&x, &u1, &u2, &d0, &d1, &d2, &d3, &d4, &d5, &d6, &d7);

            SET_BEST_DEADLINE(d0, i + 0);
            SET_BEST_DEADLINE(d1, i + 1);
            SET_BEST_DEADLINE(d2, i + 2);
            SET_BEST_DEADLINE(d3, i + 3);
            SET_BEST_DEADLINE(d4, i + 4);
            SET_BEST_DEADLINE(d5, i + 5);
            SET_BEST_DEADLINE(d6, i + 6);
            SET_BEST_DEADLINE(d7, i + 7);
            i += 8;
        } else {
            sph_shabal_deadline_fast(&scoops[i * 64], gensig, &d0);
            SET_BEST_DEADLINE(d0, i);
            i++;
        }
    }
}
