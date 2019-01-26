/*
 * Parallel implementation of Shabal, using the AVX512f unit. This code
 * compiles and runs on x86 architectures, in 32-bit or 64-bit mode,
 * which possess a AVX512f-compatible SIMD unit.
 *
 *
 * (c) 2010 SAPHIR project. This software is provided 'as-is', without
 * any epxress or implied warranty. In no event will the authors be held
 * liable for any damages arising from the use of this software.
 *
 * Permission is granted to anyone to use this software for any purpose,
 * including commercial applications, and to alter it and redistribute it
 * freely, subject to no restriction.
 *
 * Technical remarks and questions can be addressed to:
 * <thomas.pornin@cryptolog.com>
 */

#include <immintrin.h>
#include <stddef.h>
#include <string.h>
#include "mshabal_512_avx512f.h"

#ifdef __cplusplus
extern "C" {
#endif

#ifdef _MSC_VER
#pragma warning(disable : 4146)
#endif

typedef mshabal_u32 u32;

#define C32(x) ((u32)x##UL)
#define T32(x) ((x)&C32(0xFFFFFFFF))
#define ROTL32(x, n) T32(((x) << (n)) | ((x) >> (32 - (n))))

static void mshabal_compress_avx512f(mshabal512_context *sc, const unsigned char *buf0,
                                     const unsigned char *buf1, const unsigned char *buf2,
                                     const unsigned char *buf3, const unsigned char *buf4,
                                     const unsigned char *buf5, const unsigned char *buf6,
                                     const unsigned char *buf7, const unsigned char *buf8,
                                     const unsigned char *buf9, const unsigned char *buf10,
                                     const unsigned char *buf11, const unsigned char *buf12,
                                     const unsigned char *buf13, const unsigned char *buf14,
                                     const unsigned char *buf15, size_t num) {
    union {
        u32 words[16 * MSHABAL512_VECTOR_SIZE];
        __m512i data[16];
    } u;
    size_t j;
    __m512i A[12], B[16], C[16];
    __m512i one;

    for (j = 0; j < 12; j++) A[j] = _mm512_loadu_si512((__m512i *)sc->state + j);
    for (j = 0; j < 16; j++) {
        B[j] = _mm512_loadu_si512((__m512i *)sc->state + j + 12);
        C[j] = _mm512_loadu_si512((__m512i *)sc->state + j + 28);
    }
    one = _mm512_set1_epi32(C32(0xFFFFFFFF));

#define M(i) _mm512_load_si512(u.data + i)

    while (num-- > 0) {
        for (j = 0; j < 16 * MSHABAL512_VECTOR_SIZE; j += MSHABAL512_VECTOR_SIZE) {
            size_t o = j / 4;
            u.words[j + 0] = *(u32 *)(buf0 + o);
            u.words[j + 1] = *(u32 *)(buf1 + o);
            u.words[j + 2] = *(u32 *)(buf2 + o);
            u.words[j + 3] = *(u32 *)(buf3 + o);
            u.words[j + 4] = *(u32 *)(buf4 + o);
            u.words[j + 5] = *(u32 *)(buf5 + o);
            u.words[j + 6] = *(u32 *)(buf6 + o);
            u.words[j + 7] = *(u32 *)(buf7 + o);
            u.words[j + 8] = *(u32 *)(buf8 + o);
            u.words[j + 9] = *(u32 *)(buf9 + o);
            u.words[j + 10] = *(u32 *)(buf10 + o);
            u.words[j + 11] = *(u32 *)(buf11 + o);
            u.words[j + 12] = *(u32 *)(buf12 + o);
            u.words[j + 13] = *(u32 *)(buf13 + o);
            u.words[j + 14] = *(u32 *)(buf14 + o);
            u.words[j + 15] = *(u32 *)(buf15 + o);
        }

        for (j = 0; j < 16; j++) B[j] = _mm512_add_epi32(B[j], M(j));

        A[0] = _mm512_xor_si512(A[0], _mm512_set1_epi32(sc->Wlow));
        A[1] = _mm512_xor_si512(A[1], _mm512_set1_epi32(sc->Whigh));

        for (j = 0; j < 16; j++)
            B[j] = _mm512_or_si512(_mm512_slli_epi32(B[j], 17), _mm512_srli_epi32(B[j], 15));

#define PP512(xa0, xa1, xb0, xb1, xb2, xb3, xc, xm)                                   \
    do {                                                                              \
        __m512i tt;                                                                   \
        tt = _mm512_or_si512(_mm512_slli_epi32(xa1, 15), _mm512_srli_epi32(xa1, 17)); \
        tt = _mm512_add_epi32(_mm512_slli_epi32(tt, 2), tt);                          \
        tt = _mm512_xor_si512(_mm512_xor_si512(xa0, tt), xc);                         \
        tt = _mm512_add_epi32(_mm512_slli_epi32(tt, 1), tt);                          \
        tt = _mm512_xor_si512(_mm512_xor_si512(tt, xb1),                              \
                              _mm512_xor_si512(_mm512_andnot_si512(xb3, xb2), xm));   \
        xa0 = tt;                                                                     \
        tt = xb0;                                                                     \
        tt = _mm512_or_si512(_mm512_slli_epi32(tt, 1), _mm512_srli_epi32(tt, 31));    \
        xb0 = _mm512_xor_si512(tt, _mm512_xor_si512(xa0, one));                       \
    } while (0)

        PP512(A[0x0], A[0xB], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M(0x0));
        PP512(A[0x1], A[0x0], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M(0x1));
        PP512(A[0x2], A[0x1], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M(0x2));
        PP512(A[0x3], A[0x2], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M(0x3));
        PP512(A[0x4], A[0x3], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M(0x4));
        PP512(A[0x5], A[0x4], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M(0x5));
        PP512(A[0x6], A[0x5], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M(0x6));
        PP512(A[0x7], A[0x6], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M(0x7));
        PP512(A[0x8], A[0x7], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M(0x8));
        PP512(A[0x9], A[0x8], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M(0x9));
        PP512(A[0xA], A[0x9], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M(0xA));
        PP512(A[0xB], A[0xA], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M(0xB));
        PP512(A[0x0], A[0xB], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M(0xC));
        PP512(A[0x1], A[0x0], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M(0xD));
        PP512(A[0x2], A[0x1], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M(0xE));
        PP512(A[0x3], A[0x2], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M(0xF));

        PP512(A[0x4], A[0x3], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M(0x0));
        PP512(A[0x5], A[0x4], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M(0x1));
        PP512(A[0x6], A[0x5], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M(0x2));
        PP512(A[0x7], A[0x6], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M(0x3));
        PP512(A[0x8], A[0x7], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M(0x4));
        PP512(A[0x9], A[0x8], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M(0x5));
        PP512(A[0xA], A[0x9], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M(0x6));
        PP512(A[0xB], A[0xA], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M(0x7));
        PP512(A[0x0], A[0xB], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M(0x8));
        PP512(A[0x1], A[0x0], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M(0x9));
        PP512(A[0x2], A[0x1], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M(0xA));
        PP512(A[0x3], A[0x2], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M(0xB));
        PP512(A[0x4], A[0x3], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M(0xC));
        PP512(A[0x5], A[0x4], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M(0xD));
        PP512(A[0x6], A[0x5], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M(0xE));
        PP512(A[0x7], A[0x6], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M(0xF));

        PP512(A[0x8], A[0x7], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M(0x0));
        PP512(A[0x9], A[0x8], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M(0x1));
        PP512(A[0xA], A[0x9], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M(0x2));
        PP512(A[0xB], A[0xA], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M(0x3));
        PP512(A[0x0], A[0xB], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M(0x4));
        PP512(A[0x1], A[0x0], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M(0x5));
        PP512(A[0x2], A[0x1], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M(0x6));
        PP512(A[0x3], A[0x2], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M(0x7));
        PP512(A[0x4], A[0x3], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M(0x8));
        PP512(A[0x5], A[0x4], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M(0x9));
        PP512(A[0x6], A[0x5], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M(0xA));
        PP512(A[0x7], A[0x6], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M(0xB));
        PP512(A[0x8], A[0x7], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M(0xC));
        PP512(A[0x9], A[0x8], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M(0xD));
        PP512(A[0xA], A[0x9], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M(0xE));
        PP512(A[0xB], A[0xA], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M(0xF));

        A[0xB] = _mm512_add_epi32(A[0xB], C[0x6]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0x5]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0x4]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0x3]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0x2]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x1]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x0]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0xF]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0xE]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0xD]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0xC]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0xB]);
        A[0xB] = _mm512_add_epi32(A[0xB], C[0xA]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0x9]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0x8]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0x7]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0x6]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x5]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x4]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0x3]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0x2]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0x1]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0x0]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0xF]);
        A[0xB] = _mm512_add_epi32(A[0xB], C[0xE]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0xD]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0xC]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0xB]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0xA]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x9]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x8]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0x7]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0x6]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0x5]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0x4]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0x3]);

#define SWAP_AND_SUB512(xb, xc, xm)    \
    do {                               \
        __m512i tmp;                   \
        tmp = xb;                      \
        xb = _mm512_sub_epi32(xc, xm); \
        xc = tmp;                      \
    } while (0)

        SWAP_AND_SUB512(B[0x0], C[0x0], M(0x0));
        SWAP_AND_SUB512(B[0x1], C[0x1], M(0x1));
        SWAP_AND_SUB512(B[0x2], C[0x2], M(0x2));
        SWAP_AND_SUB512(B[0x3], C[0x3], M(0x3));
        SWAP_AND_SUB512(B[0x4], C[0x4], M(0x4));
        SWAP_AND_SUB512(B[0x5], C[0x5], M(0x5));
        SWAP_AND_SUB512(B[0x6], C[0x6], M(0x6));
        SWAP_AND_SUB512(B[0x7], C[0x7], M(0x7));
        SWAP_AND_SUB512(B[0x8], C[0x8], M(0x8));
        SWAP_AND_SUB512(B[0x9], C[0x9], M(0x9));
        SWAP_AND_SUB512(B[0xA], C[0xA], M(0xA));
        SWAP_AND_SUB512(B[0xB], C[0xB], M(0xB));
        SWAP_AND_SUB512(B[0xC], C[0xC], M(0xC));
        SWAP_AND_SUB512(B[0xD], C[0xD], M(0xD));
        SWAP_AND_SUB512(B[0xE], C[0xE], M(0xE));
        SWAP_AND_SUB512(B[0xF], C[0xF], M(0xF));

        buf0 += 64;
        buf1 += 64;
        buf2 += 64;
        buf3 += 64;
        buf4 += 64;
        buf5 += 64;
        buf6 += 64;
        buf7 += 64;
        buf8 += 64;
        buf9 += 64;
        buf10 += 64;
        buf11 += 64;
        buf12 += 64;
        buf13 += 64;
        buf14 += 64;
        buf15 += 64;
        if (++sc->Wlow == 0) sc->Whigh++;
    }

    for (j = 0; j < 12; j++) _mm512_storeu_si512((__m512i *)sc->state + j, A[j]);
    for (j = 0; j < 16; j++) {
        _mm512_storeu_si512((__m512i *)sc->state + j + 12, B[j]);
        _mm512_storeu_si512((__m512i *)sc->state + j + 28, C[j]);
    }

#undef M
}

void mshabal_init_avx512f(mshabal512_context *sc, unsigned out_size) {
    unsigned u;

    memset(sc->state, 0, sizeof sc->state);
    memset(sc->buf0, 0, sizeof sc->buf0);
    memset(sc->buf1, 0, sizeof sc->buf1);
    memset(sc->buf2, 0, sizeof sc->buf2);
    memset(sc->buf3, 0, sizeof sc->buf3);
    memset(sc->buf4, 0, sizeof sc->buf4);
    memset(sc->buf5, 0, sizeof sc->buf5);
    memset(sc->buf6, 0, sizeof sc->buf6);
    memset(sc->buf7, 0, sizeof sc->buf7);
    memset(sc->buf8, 0, sizeof sc->buf8);
    memset(sc->buf9, 0, sizeof sc->buf9);
    memset(sc->buf10, 0, sizeof sc->buf10);
    memset(sc->buf11, 0, sizeof sc->buf11);
    memset(sc->buf12, 0, sizeof sc->buf12);
    memset(sc->buf13, 0, sizeof sc->buf13);
    memset(sc->buf14, 0, sizeof sc->buf14);
    memset(sc->buf15, 0, sizeof sc->buf15);
    for (u = 0; u < 16; u++) {
        sc->buf0[4 * u + 0] = (out_size + u);
        sc->buf0[4 * u + 1] = (out_size + u) >> 8;
        sc->buf1[4 * u + 0] = (out_size + u);
        sc->buf1[4 * u + 1] = (out_size + u) >> 8;
        sc->buf2[4 * u + 0] = (out_size + u);
        sc->buf2[4 * u + 1] = (out_size + u) >> 8;
        sc->buf3[4 * u + 0] = (out_size + u);
        sc->buf3[4 * u + 1] = (out_size + u) >> 8;
        sc->buf4[4 * u + 0] = (out_size + u);
        sc->buf4[4 * u + 1] = (out_size + u) >> 8;
        sc->buf5[4 * u + 0] = (out_size + u);
        sc->buf5[4 * u + 1] = (out_size + u) >> 8;
        sc->buf6[4 * u + 0] = (out_size + u);
        sc->buf6[4 * u + 1] = (out_size + u) >> 8;
        sc->buf7[4 * u + 0] = (out_size + u);
        sc->buf7[4 * u + 1] = (out_size + u) >> 8;
        sc->buf8[4 * u + 0] = (out_size + u);
        sc->buf8[4 * u + 1] = (out_size + u) >> 8;
        sc->buf9[4 * u + 0] = (out_size + u);
        sc->buf9[4 * u + 1] = (out_size + u) >> 8;
        sc->buf10[4 * u + 0] = (out_size + u);
        sc->buf10[4 * u + 1] = (out_size + u) >> 8;
        sc->buf11[4 * u + 0] = (out_size + u);
        sc->buf11[4 * u + 1] = (out_size + u) >> 8;
        sc->buf12[4 * u + 0] = (out_size + u);
        sc->buf12[4 * u + 1] = (out_size + u) >> 8;
        sc->buf13[4 * u + 0] = (out_size + u);
        sc->buf13[4 * u + 1] = (out_size + u) >> 8;
        sc->buf14[4 * u + 0] = (out_size + u);
        sc->buf14[4 * u + 1] = (out_size + u) >> 8;
        sc->buf15[4 * u + 0] = (out_size + u);
        sc->buf15[4 * u + 1] = (out_size + u) >> 8;
    }
    sc->Whigh = sc->Wlow = C32(0xFFFFFFFF);
    mshabal_compress_avx512f(sc, sc->buf0, sc->buf1, sc->buf2, sc->buf3, sc->buf4, sc->buf5,
                             sc->buf6, sc->buf7, sc->buf8, sc->buf9, sc->buf10, sc->buf11,
                             sc->buf12, sc->buf13, sc->buf14, sc->buf15, 1);
    for (u = 0; u < 16; u++) {
        sc->buf0[4 * u + 0] = (out_size + u + 16);
        sc->buf0[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf1[4 * u + 0] = (out_size + u + 16);
        sc->buf1[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf2[4 * u + 0] = (out_size + u + 16);
        sc->buf2[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf3[4 * u + 0] = (out_size + u + 16);
        sc->buf3[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf4[4 * u + 0] = (out_size + u + 16);
        sc->buf4[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf5[4 * u + 0] = (out_size + u + 16);
        sc->buf5[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf6[4 * u + 0] = (out_size + u + 16);
        sc->buf6[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf7[4 * u + 0] = (out_size + u + 16);
        sc->buf7[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf8[4 * u + 0] = (out_size + u + 16);
        sc->buf8[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf9[4 * u + 0] = (out_size + u + 16);
        sc->buf9[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf10[4 * u + 0] = (out_size + u + 16);
        sc->buf10[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf11[4 * u + 0] = (out_size + u + 16);
        sc->buf11[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf12[4 * u + 0] = (out_size + u + 16);
        sc->buf12[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf13[4 * u + 0] = (out_size + u + 16);
        sc->buf13[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf14[4 * u + 0] = (out_size + u + 16);
        sc->buf14[4 * u + 1] = (out_size + u + 16) >> 8;
        sc->buf15[4 * u + 0] = (out_size + u + 16);
        sc->buf15[4 * u + 1] = (out_size + u + 16) >> 8;
    }
    mshabal_compress_avx512f(sc, sc->buf0, sc->buf1, sc->buf2, sc->buf3, sc->buf4, sc->buf5,
                             sc->buf6, sc->buf7, sc->buf8, sc->buf9, sc->buf10, sc->buf11,
                             sc->buf12, sc->buf13, sc->buf14, sc->buf15, 1);
    sc->ptr = 0;
    sc->out_size = out_size;
}

void mshabal_avx512f(mshabal512_context *sc, const void *data0, const void *data1, const void *data2,
                             const void *data3, const void *data4, const void *data5, const void *data6, const void *data7,
                             const void *data8, const void *data9, const void *data10, const void *data11, const void *data12,
                             const void *data13, const void *data14, const void *data15, size_t len) {
    size_t ptr, num;

    if (data0 == NULL) {
        if (data1 == NULL) {
            if (data2 == NULL) {
                if (data3 == NULL) {
                    if (data4 == NULL) {
                        if (data5 == NULL) {
                            if (data6 == NULL) {
                                if (data7 == NULL) {
                                    if (data8 == NULL) {
                                        if (data9 == NULL) {
                                            if (data10 == NULL) {
                                                if (data11 == NULL) {
                                                    if (data12 == NULL) {
                                                        if (data13 == NULL) {
                                                            if (data14 == NULL) {
                                                                if (data15 == NULL) {
                                                                    return;
                                                                } else {
                                                                    data0 = data15;
                                                                }
                                                            } else {
                                                                data0 = data14;
                                                            }
                                                        } else {
                                                            data0 = data13;
                                                        }
                                                    } else {
                                                        data0 = data12;
                                                    }
                                                } else {
                                                    data0 = data11;
                                                }
                                            } else {
                                                data0 = data10;
                                            }
                                        } else {
                                            data0 = data9;
                                        }
                                    } else {
                                        data0 = data8;
                                    }
                                } else {
                                    data0 = data7;
                                }
                            } else {
                                data0 = data6;
                            }
                        } else {
                            data0 = data5;
                        }
                    } else {
                        data0 = data4;
                    }
                } else {
                    data0 = data3;
                }
            } else {
                data0 = data2;
            }
        } else {
            data0 = data1;
        }
    }

    if (data1 == NULL) data1 = data0;
    if (data2 == NULL) data2 = data0;
    if (data3 == NULL) data3 = data0;
    if (data4 == NULL) data4 = data0;
    if (data5 == NULL) data5 = data0;
    if (data6 == NULL) data6 = data0;
    if (data7 == NULL) data7 = data0;
    if (data8 == NULL) data8 = data0;
    if (data9 == NULL) data9 = data0;
    if (data10 == NULL) data10 = data0;
    if (data11 == NULL) data11 = data0;
    if (data12 == NULL) data12 = data0;
    if (data13 == NULL) data13 = data0;
    if (data14 == NULL) data14 = data0;
    if (data15 == NULL) data15 = data0;

    ptr = sc->ptr;
    if (ptr != 0) {
        size_t clen = (sizeof sc->buf0 - ptr);
        if (clen > len) {
            memcpy(sc->buf0 + ptr, data0, len);
            memcpy(sc->buf1 + ptr, data1, len);
            memcpy(sc->buf2 + ptr, data2, len);
            memcpy(sc->buf3 + ptr, data3, len);
            memcpy(sc->buf4 + ptr, data4, len);
            memcpy(sc->buf5 + ptr, data5, len);
            memcpy(sc->buf6 + ptr, data6, len);
            memcpy(sc->buf7 + ptr, data7, len);
            memcpy(sc->buf8 + ptr, data8, len);
            memcpy(sc->buf9 + ptr, data9, len);
            memcpy(sc->buf10 + ptr, data10, len);
            memcpy(sc->buf11 + ptr, data11, len);
            memcpy(sc->buf12 + ptr, data12, len);
            memcpy(sc->buf13 + ptr, data13, len);
            memcpy(sc->buf14 + ptr, data14, len);
            memcpy(sc->buf15 + ptr, data15, len);
            sc->ptr = ptr + len;
            return;
        } else {
            memcpy(sc->buf0 + ptr, data0, clen);
            memcpy(sc->buf1 + ptr, data1, clen);
            memcpy(sc->buf2 + ptr, data2, clen);
            memcpy(sc->buf3 + ptr, data3, clen);
            memcpy(sc->buf4 + ptr, data4, clen);
            memcpy(sc->buf5 + ptr, data5, clen);
            memcpy(sc->buf6 + ptr, data6, clen);
            memcpy(sc->buf7 + ptr, data7, clen);
            memcpy(sc->buf8 + ptr, data8, clen);
            memcpy(sc->buf9 + ptr, data9, clen);
            memcpy(sc->buf10 + ptr, data10, clen);
            memcpy(sc->buf11 + ptr, data11, clen);
            memcpy(sc->buf12 + ptr, data12, clen);
            memcpy(sc->buf13 + ptr, data13, clen);
            memcpy(sc->buf14 + ptr, data14, clen);
            memcpy(sc->buf15 + ptr, data15, clen);
            mshabal_compress_avx512f(sc, sc->buf0, sc->buf1, sc->buf2, sc->buf3, sc->buf4, sc->buf5,
                                     sc->buf6, sc->buf7, sc->buf8, sc->buf9, sc->buf10, sc->buf11,
                                     sc->buf12, sc->buf13, sc->buf14, sc->buf15, 1);
            data0 = (const unsigned char *)data0 + clen;
            data1 = (const unsigned char *)data1 + clen;
            data2 = (const unsigned char *)data2 + clen;
            data3 = (const unsigned char *)data3 + clen;
            data4 = (const unsigned char *)data4 + clen;
            data5 = (const unsigned char *)data5 + clen;
            data6 = (const unsigned char *)data6 + clen;
            data7 = (const unsigned char *)data7 + clen;
            data8 = (const unsigned char *)data8 + clen;
            data9 = (const unsigned char *)data9 + clen;
            data10 = (const unsigned char *)data10 + clen;
            data11 = (const unsigned char *)data11 + clen;
            data12 = (const unsigned char *)data12 + clen;
            data13 = (const unsigned char *)data13 + clen;
            data14 = (const unsigned char *)data14 + clen;
            data15 = (const unsigned char *)data15 + clen;
            len -= clen;
        }
    }

    num = len >> 6;
    if (num != 0) {
       mshabal_compress_avx512f(sc, data0, data1, data2, data3, data4, data5, data6, data7,
                                    data8, data9, data10, data11, data12, data13, data14, data15, num);
        data0 = (const unsigned char *)data0 + (num << 6);
        data1 = (const unsigned char *)data1 + (num << 6);
        data2 = (const unsigned char *)data2 + (num << 6);
        data3 = (const unsigned char *)data3 + (num << 6);
        data4 = (const unsigned char *)data4 + (num << 6);
        data5 = (const unsigned char *)data5 + (num << 6);
        data6 = (const unsigned char *)data6 + (num << 6);
        data7 = (const unsigned char *)data7 + (num << 6);
        data8 = (const unsigned char *)data8 + (num << 6);
        data9 = (const unsigned char *)data9 + (num << 6);
        data10 = (const unsigned char *)data10 + (num << 6);
        data11 = (const unsigned char *)data11 + (num << 6);
        data12 = (const unsigned char *)data12 + (num << 6);
        data13 = (const unsigned char *)data13 + (num << 6);
        data14 = (const unsigned char *)data14 + (num << 6);
        data15 = (const unsigned char *)data15 + (num << 6);
    }
    len &= (size_t)63;
    memcpy(sc->buf0, data0, len);
    memcpy(sc->buf1, data1, len);
    memcpy(sc->buf2, data2, len);
    memcpy(sc->buf3, data3, len);
    memcpy(sc->buf4, data4, len);
    memcpy(sc->buf5, data5, len);
    memcpy(sc->buf6, data6, len);
    memcpy(sc->buf7, data7, len);
    memcpy(sc->buf8, data8, len);
    memcpy(sc->buf9, data9, len);
    memcpy(sc->buf10, data10, len);
    memcpy(sc->buf11, data11, len);
    memcpy(sc->buf12, data12, len);
    memcpy(sc->buf13, data13, len);
    memcpy(sc->buf14, data14, len);
    memcpy(sc->buf15, data15, len);
    sc->ptr = len;
}

void mshabal_close_avx512f(mshabal512_context *sc, unsigned ub0, unsigned ub1, unsigned ub2,
                      unsigned ub3, unsigned ub4, unsigned ub5, unsigned ub6, unsigned ub7, 
                      unsigned ub8, unsigned ub9, unsigned ub10, unsigned ub11, unsigned ub12, 
                      unsigned ub13, unsigned ub14, unsigned ub15,
                      unsigned n, void *dst0, void *dst1, void *dst2, void *dst3, void *dst4,
                      void *dst5, void *dst6, void *dst7, void *dst8, void *dst9, void *dst10, 
                      void *dst11, void *dst12, void *dst13, void *dst14, void *dst15) {
    size_t ptr, off;
    unsigned z, out_size_w32;

    z = 0x80 >> n;
    ptr = sc->ptr;
    sc->buf0[ptr] = (ub0 & -z) | z;
    sc->buf1[ptr] = (ub1 & -z) | z;
    sc->buf2[ptr] = (ub2 & -z) | z;
    sc->buf3[ptr] = (ub3 & -z) | z;
    sc->buf4[ptr] = (ub4 & -z) | z;
    sc->buf5[ptr] = (ub5 & -z) | z;
    sc->buf6[ptr] = (ub6 & -z) | z;
    sc->buf7[ptr] = (ub7 & -z) | z;
    sc->buf8[ptr] = (ub8 & -z) | z;
    sc->buf9[ptr] = (ub9 & -z) | z;
    sc->buf10[ptr] = (ub10 & -z) | z;
    sc->buf11[ptr] = (ub11 & -z) | z;
    sc->buf12[ptr] = (ub12 & -z) | z;
    sc->buf13[ptr] = (ub13 & -z) | z;
    sc->buf14[ptr] = (ub14 & -z) | z;
    sc->buf15[ptr] = (ub15 & -z) | z;
    ptr++;
    memset(sc->buf0 + ptr, 0, (sizeof sc->buf0) - ptr);
    memset(sc->buf1 + ptr, 0, (sizeof sc->buf1) - ptr);
    memset(sc->buf2 + ptr, 0, (sizeof sc->buf2) - ptr);
    memset(sc->buf3 + ptr, 0, (sizeof sc->buf3) - ptr);
    memset(sc->buf4 + ptr, 0, (sizeof sc->buf4) - ptr);
    memset(sc->buf5 + ptr, 0, (sizeof sc->buf5) - ptr);
    memset(sc->buf6 + ptr, 0, (sizeof sc->buf6) - ptr);
    memset(sc->buf7 + ptr, 0, (sizeof sc->buf7) - ptr);
    memset(sc->buf8 + ptr, 0, (sizeof sc->buf8) - ptr);
    memset(sc->buf9 + ptr, 0, (sizeof sc->buf9) - ptr);
    memset(sc->buf10 + ptr, 0, (sizeof sc->buf10) - ptr);
    memset(sc->buf11 + ptr, 0, (sizeof sc->buf11) - ptr);
    memset(sc->buf12 + ptr, 0, (sizeof sc->buf12) - ptr);
    memset(sc->buf13 + ptr, 0, (sizeof sc->buf13) - ptr);
    memset(sc->buf14 + ptr, 0, (sizeof sc->buf14) - ptr);
    memset(sc->buf15 + ptr, 0, (sizeof sc->buf15) - ptr);
    for (z = 0; z < 4; z++) {
        mshabal_compress_avx512f(sc, sc->buf0, sc->buf1, sc->buf2, sc->buf3, sc->buf4, sc->buf5,
                            sc->buf6, sc->buf7, sc->buf8, sc->buf9, sc->buf10, sc->buf11, sc->buf12, sc->buf13,
                            sc->buf14, sc->buf15, 1);
        if (sc->Wlow-- == 0) sc->Whigh--;
    }
    out_size_w32 = sc->out_size >> 5;
    off = MSHABAL512_VECTOR_SIZE * (28 + (16 - out_size_w32));
    if (dst0 != NULL) {
        u32 *out;

        out = (u32 *)dst0;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 0];
    }
    if (dst1 != NULL) {
        u32 *out;

        out = (u32 *)dst1;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 1];
    }
    if (dst2 != NULL) {
        u32 *out;

        out = (u32 *)dst2;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 2];
    }
    if (dst3 != NULL) {
        u32 *out;

        out = (u32 *)dst3;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 3];
    }
    if (dst4 != NULL) {
        u32 *out;

        out = (u32 *)dst4;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 4];
    }
    if (dst5 != NULL) {
        u32 *out;

        out = (u32 *)dst5;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 5];
    }
    if (dst6 != NULL) {
        u32 *out;

        out = (u32 *)dst6;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 6];
    }
    if (dst7 != NULL) {
        u32 *out;

        out = (u32 *)dst7;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 7];
    }
    if (dst8 != NULL) {
        u32 *out;

        out = (u32 *)dst8;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 8];
    }
    if (dst9 != NULL) {
        u32 *out;

        out = (u32 *)dst9;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 9];
    }
    if (dst10 != NULL) {
        u32 *out;

        out = (u32 *)dst10;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 10];
    }
    if (dst11 != NULL) {
        u32 *out;

        out = (u32 *)dst11;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 11];
    }
    if (dst12 != NULL) {
        u32 *out;

        out = (u32 *)dst12;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 12];
    }
    if (dst13 != NULL) {
        u32 *out;

        out = (u32 *)dst13;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 13];
    }
    if (dst14 != NULL) {
        u32 *out;

        out = (u32 *)dst14;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 14];
    }
    if (dst15 != NULL) {
        u32 *out;

        out = (u32 *)dst15;
        for (z = 0; z < out_size_w32; z++)
            out[z] = sc->state[off + z * MSHABAL512_VECTOR_SIZE + 15];
    }
}

// Shabal routine optimized for plotting and hashing
void mshabal_hash_fast_avx512f(mshabal512_context_fast *sc, void *message, void *termination,
                                    void *dst, unsigned num) {
    union input {
        u32 words[16 * MSHABAL512_VECTOR_SIZE];
        __m512i data[16];
    };
    size_t j;
    __m512i A[12], B[16], C[16];
    __m512i one;

    for (j = 0; j < 12; j++) A[j] = _mm512_loadu_si512((__m512i *)sc->state + j);
    for (j = 0; j < 16; j++) {
        B[j] = _mm512_loadu_si512((__m512i *)sc->state + j + 12);
        C[j] = _mm512_loadu_si512((__m512i *)sc->state + j + 28);
    }
    one = _mm512_set1_epi32(C32(0xFFFFFFFF));

    // round 1
#define M(i) _mm512_load_si512((__m512i *)message + i)

    while (num-- > 0) {
        for (j = 0; j < 16; j++) B[j] = _mm512_add_epi32(B[j], M(j));

        A[0] = _mm512_xor_si512(A[0], _mm512_set1_epi32(sc->Wlow));
        A[1] = _mm512_xor_si512(A[1], _mm512_set1_epi32(sc->Whigh));

        for (j = 0; j < 16; j++)
            B[j] = _mm512_or_si512(_mm512_slli_epi32(B[j], 17), _mm512_srli_epi32(B[j], 15));

#define PP512(xa0, xa1, xb0, xb1, xb2, xb3, xc, xm)                                   \
    do {                                                                              \
        __m512i tt;                                                                   \
        tt = _mm512_or_si512(_mm512_slli_epi32(xa1, 15), _mm512_srli_epi32(xa1, 17)); \
        tt = _mm512_add_epi32(_mm512_slli_epi32(tt, 2), tt);                          \
        tt = _mm512_xor_si512(_mm512_xor_si512(xa0, tt), xc);                         \
        tt = _mm512_add_epi32(_mm512_slli_epi32(tt, 1), tt);                          \
        tt = _mm512_xor_si512(_mm512_xor_si512(tt, xb1),                              \
                              _mm512_xor_si512(_mm512_andnot_si512(xb3, xb2), xm));   \
        xa0 = tt;                                                                     \
        tt = xb0;                                                                     \
        tt = _mm512_or_si512(_mm512_slli_epi32(tt, 1), _mm512_srli_epi32(tt, 31));    \
        xb0 = _mm512_xor_si512(tt, _mm512_xor_si512(xa0, one));                       \
    } while (0)

        PP512(A[0x0], A[0xB], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M(0x0));
        PP512(A[0x1], A[0x0], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M(0x1));
        PP512(A[0x2], A[0x1], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M(0x2));
        PP512(A[0x3], A[0x2], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M(0x3));
        PP512(A[0x4], A[0x3], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M(0x4));
        PP512(A[0x5], A[0x4], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M(0x5));
        PP512(A[0x6], A[0x5], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M(0x6));
        PP512(A[0x7], A[0x6], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M(0x7));
        PP512(A[0x8], A[0x7], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M(0x8));
        PP512(A[0x9], A[0x8], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M(0x9));
        PP512(A[0xA], A[0x9], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M(0xA));
        PP512(A[0xB], A[0xA], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M(0xB));
        PP512(A[0x0], A[0xB], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M(0xC));
        PP512(A[0x1], A[0x0], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M(0xD));
        PP512(A[0x2], A[0x1], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M(0xE));
        PP512(A[0x3], A[0x2], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M(0xF));

        PP512(A[0x4], A[0x3], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M(0x0));
        PP512(A[0x5], A[0x4], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M(0x1));
        PP512(A[0x6], A[0x5], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M(0x2));
        PP512(A[0x7], A[0x6], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M(0x3));
        PP512(A[0x8], A[0x7], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M(0x4));
        PP512(A[0x9], A[0x8], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M(0x5));
        PP512(A[0xA], A[0x9], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M(0x6));
        PP512(A[0xB], A[0xA], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M(0x7));
        PP512(A[0x0], A[0xB], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M(0x8));
        PP512(A[0x1], A[0x0], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M(0x9));
        PP512(A[0x2], A[0x1], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M(0xA));
        PP512(A[0x3], A[0x2], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M(0xB));
        PP512(A[0x4], A[0x3], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M(0xC));
        PP512(A[0x5], A[0x4], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M(0xD));
        PP512(A[0x6], A[0x5], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M(0xE));
        PP512(A[0x7], A[0x6], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M(0xF));

        PP512(A[0x8], A[0x7], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M(0x0));
        PP512(A[0x9], A[0x8], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M(0x1));
        PP512(A[0xA], A[0x9], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M(0x2));
        PP512(A[0xB], A[0xA], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M(0x3));
        PP512(A[0x0], A[0xB], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M(0x4));
        PP512(A[0x1], A[0x0], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M(0x5));
        PP512(A[0x2], A[0x1], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M(0x6));
        PP512(A[0x3], A[0x2], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M(0x7));
        PP512(A[0x4], A[0x3], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M(0x8));
        PP512(A[0x5], A[0x4], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M(0x9));
        PP512(A[0x6], A[0x5], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M(0xA));
        PP512(A[0x7], A[0x6], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M(0xB));
        PP512(A[0x8], A[0x7], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M(0xC));
        PP512(A[0x9], A[0x8], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M(0xD));
        PP512(A[0xA], A[0x9], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M(0xE));
        PP512(A[0xB], A[0xA], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M(0xF));

        A[0xB] = _mm512_add_epi32(A[0xB], C[0x6]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0x5]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0x4]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0x3]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0x2]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x1]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x0]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0xF]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0xE]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0xD]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0xC]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0xB]);
        A[0xB] = _mm512_add_epi32(A[0xB], C[0xA]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0x9]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0x8]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0x7]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0x6]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x5]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x4]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0x3]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0x2]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0x1]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0x0]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0xF]);
        A[0xB] = _mm512_add_epi32(A[0xB], C[0xE]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0xD]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0xC]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0xB]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0xA]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x9]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x8]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0x7]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0x6]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0x5]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0x4]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0x3]);

#define SWAP_AND_SUB512(xb, xc, xm)    \
    do {                               \
        __m512i tmp;                   \
        tmp = xb;                      \
        xb = _mm512_sub_epi32(xc, xm); \
        xc = tmp;                      \
    } while (0)

        SWAP_AND_SUB512(B[0x0], C[0x0], M(0x0));
        SWAP_AND_SUB512(B[0x1], C[0x1], M(0x1));
        SWAP_AND_SUB512(B[0x2], C[0x2], M(0x2));
        SWAP_AND_SUB512(B[0x3], C[0x3], M(0x3));
        SWAP_AND_SUB512(B[0x4], C[0x4], M(0x4));
        SWAP_AND_SUB512(B[0x5], C[0x5], M(0x5));
        SWAP_AND_SUB512(B[0x6], C[0x6], M(0x6));
        SWAP_AND_SUB512(B[0x7], C[0x7], M(0x7));
        SWAP_AND_SUB512(B[0x8], C[0x8], M(0x8));
        SWAP_AND_SUB512(B[0x9], C[0x9], M(0x9));
        SWAP_AND_SUB512(B[0xA], C[0xA], M(0xA));
        SWAP_AND_SUB512(B[0xB], C[0xB], M(0xB));
        SWAP_AND_SUB512(B[0xC], C[0xC], M(0xC));
        SWAP_AND_SUB512(B[0xD], C[0xD], M(0xD));
        SWAP_AND_SUB512(B[0xE], C[0xE], M(0xE));
        SWAP_AND_SUB512(B[0xF], C[0xF], M(0xF));

        // move data pointer
        message = (__m512i *)message + 16;

        if (++sc->Wlow == 0) sc->Whigh++;
    }
    
    // round 2-5
#define M2(i) _mm512_load_si512((__m512i *)termination + i)

    for (int k = 0; k < 4; k++) {
        for (j = 0; j < 16; j++) B[j] = _mm512_add_epi32(B[j], M2(j));

        A[0] = _mm512_xor_si512(A[0], _mm512_set1_epi32(sc->Wlow));
        A[1] = _mm512_xor_si512(A[1], _mm512_set1_epi32(sc->Whigh));

        for (j = 0; j < 16; j++)
            B[j] = _mm512_or_si512(_mm512_slli_epi32(B[j], 17), _mm512_srli_epi32(B[j], 15));

        PP512(A[0x0], A[0xB], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M2(0x0));
        PP512(A[0x1], A[0x0], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M2(0x1));
        PP512(A[0x2], A[0x1], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M2(0x2));
        PP512(A[0x3], A[0x2], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M2(0x3));
        PP512(A[0x4], A[0x3], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M2(0x4));
        PP512(A[0x5], A[0x4], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M2(0x5));
        PP512(A[0x6], A[0x5], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M2(0x6));
        PP512(A[0x7], A[0x6], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M2(0x7));
        PP512(A[0x8], A[0x7], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M2(0x8));
        PP512(A[0x9], A[0x8], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M2(0x9));
        PP512(A[0xA], A[0x9], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M2(0xA));
        PP512(A[0xB], A[0xA], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M2(0xB));
        PP512(A[0x0], A[0xB], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M2(0xC));
        PP512(A[0x1], A[0x0], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M2(0xD));
        PP512(A[0x2], A[0x1], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M2(0xE));
        PP512(A[0x3], A[0x2], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M2(0xF));

        PP512(A[0x4], A[0x3], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M2(0x0));
        PP512(A[0x5], A[0x4], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M2(0x1));
        PP512(A[0x6], A[0x5], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M2(0x2));
        PP512(A[0x7], A[0x6], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M2(0x3));
        PP512(A[0x8], A[0x7], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M2(0x4));
        PP512(A[0x9], A[0x8], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M2(0x5));
        PP512(A[0xA], A[0x9], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M2(0x6));
        PP512(A[0xB], A[0xA], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M2(0x7));
        PP512(A[0x0], A[0xB], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M2(0x8));
        PP512(A[0x1], A[0x0], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M2(0x9));
        PP512(A[0x2], A[0x1], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M2(0xA));
        PP512(A[0x3], A[0x2], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M2(0xB));
        PP512(A[0x4], A[0x3], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M2(0xC));
        PP512(A[0x5], A[0x4], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M2(0xD));
        PP512(A[0x6], A[0x5], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M2(0xE));
        PP512(A[0x7], A[0x6], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M2(0xF));

        PP512(A[0x8], A[0x7], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M2(0x0));
        PP512(A[0x9], A[0x8], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M2(0x1));
        PP512(A[0xA], A[0x9], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M2(0x2));
        PP512(A[0xB], A[0xA], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M2(0x3));
        PP512(A[0x0], A[0xB], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M2(0x4));
        PP512(A[0x1], A[0x0], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M2(0x5));
        PP512(A[0x2], A[0x1], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M2(0x6));
        PP512(A[0x3], A[0x2], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M2(0x7));
        PP512(A[0x4], A[0x3], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M2(0x8));
        PP512(A[0x5], A[0x4], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M2(0x9));
        PP512(A[0x6], A[0x5], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M2(0xA));
        PP512(A[0x7], A[0x6], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M2(0xB));
        PP512(A[0x8], A[0x7], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M2(0xC));
        PP512(A[0x9], A[0x8], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M2(0xD));
        PP512(A[0xA], A[0x9], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M2(0xE));
        PP512(A[0xB], A[0xA], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M2(0xF));

        A[0xB] = _mm512_add_epi32(A[0xB], C[0x6]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0x5]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0x4]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0x3]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0x2]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x1]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x0]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0xF]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0xE]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0xD]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0xC]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0xB]);
        A[0xB] = _mm512_add_epi32(A[0xB], C[0xA]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0x9]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0x8]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0x7]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0x6]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x5]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x4]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0x3]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0x2]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0x1]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0x0]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0xF]);
        A[0xB] = _mm512_add_epi32(A[0xB], C[0xE]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0xD]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0xC]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0xB]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0xA]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x9]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x8]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0x7]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0x6]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0x5]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0x4]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0x3]);

        SWAP_AND_SUB512(B[0x0], C[0x0], M2(0x0));
        SWAP_AND_SUB512(B[0x1], C[0x1], M2(0x1));
        SWAP_AND_SUB512(B[0x2], C[0x2], M2(0x2));
        SWAP_AND_SUB512(B[0x3], C[0x3], M2(0x3));
        SWAP_AND_SUB512(B[0x4], C[0x4], M2(0x4));
        SWAP_AND_SUB512(B[0x5], C[0x5], M2(0x5));
        SWAP_AND_SUB512(B[0x6], C[0x6], M2(0x6));
        SWAP_AND_SUB512(B[0x7], C[0x7], M2(0x7));
        SWAP_AND_SUB512(B[0x8], C[0x8], M2(0x8));
        SWAP_AND_SUB512(B[0x9], C[0x9], M2(0x9));
        SWAP_AND_SUB512(B[0xA], C[0xA], M2(0xA));
        SWAP_AND_SUB512(B[0xB], C[0xB], M2(0xB));
        SWAP_AND_SUB512(B[0xC], C[0xC], M2(0xC));
        SWAP_AND_SUB512(B[0xD], C[0xD], M2(0xD));
        SWAP_AND_SUB512(B[0xE], C[0xE], M2(0xE));
        SWAP_AND_SUB512(B[0xF], C[0xF], M2(0xF));

        if (++sc->Wlow == 0) sc->Whigh++;

        if (sc->Wlow-- == 0) sc->Whigh--;
    }

    // download SIMD aligned hashes
    for (j = 0; j < 8; j++) {
        _mm512_storeu_si512((__m512i *)dst + j, C[j+8]);
    }

    // reset Wlow & Whigh
    sc->Wlow = 1;
    sc->Whigh = 0;
}

// Shabal routine optimized for mining
void mshabal_deadline_fast_avx512f(mshabal512_context_fast *sc, void *message, void *termination, void *dst0,
                                    void *dst1, void *dst2, void *dst3, void *dst4, void *dst5,
                                    void *dst6, void *dst7, void *dst8, void *dst9, void *dst10,
                                    void *dst11, void *dst12, void *dst13, void *dst14,
                                    void *dst15) {
    union input {
        u32 words[16 * MSHABAL512_VECTOR_SIZE];
        __m512i data[16];
    };
    size_t j;
    __m512i A[12], B[16], C[16];
    __m512i one;

    for (j = 0; j < 12; j++) A[j] = _mm512_loadu_si512((__m512i *)sc->state + j);
    for (j = 0; j < 16; j++) {
        B[j] = _mm512_loadu_si512((__m512i *)sc->state + j + 12);
        C[j] = _mm512_loadu_si512((__m512i *)sc->state + j + 28);
    }
    one = _mm512_set1_epi32(C32(0xFFFFFFFF));

    // round 1
#define M(i) _mm512_load_si512((__m512i *)message + i)

    for (j = 0; j < 16; j++) B[j] = _mm512_add_epi32(B[j], M(j));

    A[0] = _mm512_xor_si512(A[0], _mm512_set1_epi32(sc->Wlow));
    A[1] = _mm512_xor_si512(A[1], _mm512_set1_epi32(sc->Whigh));

    for (j = 0; j < 16; j++)
        B[j] = _mm512_or_si512(_mm512_slli_epi32(B[j], 17), _mm512_srli_epi32(B[j], 15));

#define PP512(xa0, xa1, xb0, xb1, xb2, xb3, xc, xm)                                   \
    do {                                                                              \
        __m512i tt;                                                                   \
        tt = _mm512_or_si512(_mm512_slli_epi32(xa1, 15), _mm512_srli_epi32(xa1, 17)); \
        tt = _mm512_add_epi32(_mm512_slli_epi32(tt, 2), tt);                          \
        tt = _mm512_xor_si512(_mm512_xor_si512(xa0, tt), xc);                         \
        tt = _mm512_add_epi32(_mm512_slli_epi32(tt, 1), tt);                          \
        tt = _mm512_xor_si512(_mm512_xor_si512(tt, xb1),                              \
                              _mm512_xor_si512(_mm512_andnot_si512(xb3, xb2), xm));   \
        xa0 = tt;                                                                     \
        tt = xb0;                                                                     \
        tt = _mm512_or_si512(_mm512_slli_epi32(tt, 1), _mm512_srli_epi32(tt, 31));    \
        xb0 = _mm512_xor_si512(tt, _mm512_xor_si512(xa0, one));                       \
    } while (0)

    PP512(A[0x0], A[0xB], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M(0x0));
    PP512(A[0x1], A[0x0], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M(0x1));
    PP512(A[0x2], A[0x1], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M(0x2));
    PP512(A[0x3], A[0x2], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M(0x3));
    PP512(A[0x4], A[0x3], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M(0x4));
    PP512(A[0x5], A[0x4], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M(0x5));
    PP512(A[0x6], A[0x5], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M(0x6));
    PP512(A[0x7], A[0x6], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M(0x7));
    PP512(A[0x8], A[0x7], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M(0x8));
    PP512(A[0x9], A[0x8], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M(0x9));
    PP512(A[0xA], A[0x9], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M(0xA));
    PP512(A[0xB], A[0xA], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M(0xB));
    PP512(A[0x0], A[0xB], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M(0xC));
    PP512(A[0x1], A[0x0], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M(0xD));
    PP512(A[0x2], A[0x1], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M(0xE));
    PP512(A[0x3], A[0x2], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M(0xF));

    PP512(A[0x4], A[0x3], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M(0x0));
    PP512(A[0x5], A[0x4], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M(0x1));
    PP512(A[0x6], A[0x5], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M(0x2));
    PP512(A[0x7], A[0x6], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M(0x3));
    PP512(A[0x8], A[0x7], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M(0x4));
    PP512(A[0x9], A[0x8], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M(0x5));
    PP512(A[0xA], A[0x9], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M(0x6));
    PP512(A[0xB], A[0xA], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M(0x7));
    PP512(A[0x0], A[0xB], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M(0x8));
    PP512(A[0x1], A[0x0], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M(0x9));
    PP512(A[0x2], A[0x1], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M(0xA));
    PP512(A[0x3], A[0x2], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M(0xB));
    PP512(A[0x4], A[0x3], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M(0xC));
    PP512(A[0x5], A[0x4], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M(0xD));
    PP512(A[0x6], A[0x5], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M(0xE));
    PP512(A[0x7], A[0x6], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M(0xF));

    PP512(A[0x8], A[0x7], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M(0x0));
    PP512(A[0x9], A[0x8], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M(0x1));
    PP512(A[0xA], A[0x9], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M(0x2));
    PP512(A[0xB], A[0xA], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M(0x3));
    PP512(A[0x0], A[0xB], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M(0x4));
    PP512(A[0x1], A[0x0], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M(0x5));
    PP512(A[0x2], A[0x1], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M(0x6));
    PP512(A[0x3], A[0x2], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M(0x7));
    PP512(A[0x4], A[0x3], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M(0x8));
    PP512(A[0x5], A[0x4], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M(0x9));
    PP512(A[0x6], A[0x5], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M(0xA));
    PP512(A[0x7], A[0x6], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M(0xB));
    PP512(A[0x8], A[0x7], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M(0xC));
    PP512(A[0x9], A[0x8], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M(0xD));
    PP512(A[0xA], A[0x9], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M(0xE));
    PP512(A[0xB], A[0xA], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M(0xF));

    A[0xB] = _mm512_add_epi32(A[0xB], C[0x6]);
    A[0xA] = _mm512_add_epi32(A[0xA], C[0x5]);
    A[0x9] = _mm512_add_epi32(A[0x9], C[0x4]);
    A[0x8] = _mm512_add_epi32(A[0x8], C[0x3]);
    A[0x7] = _mm512_add_epi32(A[0x7], C[0x2]);
    A[0x6] = _mm512_add_epi32(A[0x6], C[0x1]);
    A[0x5] = _mm512_add_epi32(A[0x5], C[0x0]);
    A[0x4] = _mm512_add_epi32(A[0x4], C[0xF]);
    A[0x3] = _mm512_add_epi32(A[0x3], C[0xE]);
    A[0x2] = _mm512_add_epi32(A[0x2], C[0xD]);
    A[0x1] = _mm512_add_epi32(A[0x1], C[0xC]);
    A[0x0] = _mm512_add_epi32(A[0x0], C[0xB]);
    A[0xB] = _mm512_add_epi32(A[0xB], C[0xA]);
    A[0xA] = _mm512_add_epi32(A[0xA], C[0x9]);
    A[0x9] = _mm512_add_epi32(A[0x9], C[0x8]);
    A[0x8] = _mm512_add_epi32(A[0x8], C[0x7]);
    A[0x7] = _mm512_add_epi32(A[0x7], C[0x6]);
    A[0x6] = _mm512_add_epi32(A[0x6], C[0x5]);
    A[0x5] = _mm512_add_epi32(A[0x5], C[0x4]);
    A[0x4] = _mm512_add_epi32(A[0x4], C[0x3]);
    A[0x3] = _mm512_add_epi32(A[0x3], C[0x2]);
    A[0x2] = _mm512_add_epi32(A[0x2], C[0x1]);
    A[0x1] = _mm512_add_epi32(A[0x1], C[0x0]);
    A[0x0] = _mm512_add_epi32(A[0x0], C[0xF]);
    A[0xB] = _mm512_add_epi32(A[0xB], C[0xE]);
    A[0xA] = _mm512_add_epi32(A[0xA], C[0xD]);
    A[0x9] = _mm512_add_epi32(A[0x9], C[0xC]);
    A[0x8] = _mm512_add_epi32(A[0x8], C[0xB]);
    A[0x7] = _mm512_add_epi32(A[0x7], C[0xA]);
    A[0x6] = _mm512_add_epi32(A[0x6], C[0x9]);
    A[0x5] = _mm512_add_epi32(A[0x5], C[0x8]);
    A[0x4] = _mm512_add_epi32(A[0x4], C[0x7]);
    A[0x3] = _mm512_add_epi32(A[0x3], C[0x6]);
    A[0x2] = _mm512_add_epi32(A[0x2], C[0x5]);
    A[0x1] = _mm512_add_epi32(A[0x1], C[0x4]);
    A[0x0] = _mm512_add_epi32(A[0x0], C[0x3]);

#define SWAP_AND_SUB512(xb, xc, xm)    \
    do {                               \
        __m512i tmp;                   \
        tmp = xb;                      \
        xb = _mm512_sub_epi32(xc, xm); \
        xc = tmp;                      \
    } while (0)

    SWAP_AND_SUB512(B[0x0], C[0x0], M(0x0));
    SWAP_AND_SUB512(B[0x1], C[0x1], M(0x1));
    SWAP_AND_SUB512(B[0x2], C[0x2], M(0x2));
    SWAP_AND_SUB512(B[0x3], C[0x3], M(0x3));
    SWAP_AND_SUB512(B[0x4], C[0x4], M(0x4));
    SWAP_AND_SUB512(B[0x5], C[0x5], M(0x5));
    SWAP_AND_SUB512(B[0x6], C[0x6], M(0x6));
    SWAP_AND_SUB512(B[0x7], C[0x7], M(0x7));
    SWAP_AND_SUB512(B[0x8], C[0x8], M(0x8));
    SWAP_AND_SUB512(B[0x9], C[0x9], M(0x9));
    SWAP_AND_SUB512(B[0xA], C[0xA], M(0xA));
    SWAP_AND_SUB512(B[0xB], C[0xB], M(0xB));
    SWAP_AND_SUB512(B[0xC], C[0xC], M(0xC));
    SWAP_AND_SUB512(B[0xD], C[0xD], M(0xD));
    SWAP_AND_SUB512(B[0xE], C[0xE], M(0xE));
    SWAP_AND_SUB512(B[0xF], C[0xF], M(0xF));

    if (++sc->Wlow == 0) sc->Whigh++;

    // round 2-5
#define M2(i) _mm512_load_si512((__m512i *)termination + i)

    for (int k = 0; k < 4; k++) {
        for (j = 0; j < 16; j++) B[j] = _mm512_add_epi32(B[j], M2(j));

        A[0] = _mm512_xor_si512(A[0], _mm512_set1_epi32(sc->Wlow));
        A[1] = _mm512_xor_si512(A[1], _mm512_set1_epi32(sc->Whigh));

        for (j = 0; j < 16; j++)
            B[j] = _mm512_or_si512(_mm512_slli_epi32(B[j], 17), _mm512_srli_epi32(B[j], 15));

        PP512(A[0x0], A[0xB], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M2(0x0));
        PP512(A[0x1], A[0x0], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M2(0x1));
        PP512(A[0x2], A[0x1], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M2(0x2));
        PP512(A[0x3], A[0x2], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M2(0x3));
        PP512(A[0x4], A[0x3], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M2(0x4));
        PP512(A[0x5], A[0x4], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M2(0x5));
        PP512(A[0x6], A[0x5], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M2(0x6));
        PP512(A[0x7], A[0x6], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M2(0x7));
        PP512(A[0x8], A[0x7], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M2(0x8));
        PP512(A[0x9], A[0x8], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M2(0x9));
        PP512(A[0xA], A[0x9], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M2(0xA));
        PP512(A[0xB], A[0xA], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M2(0xB));
        PP512(A[0x0], A[0xB], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M2(0xC));
        PP512(A[0x1], A[0x0], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M2(0xD));
        PP512(A[0x2], A[0x1], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M2(0xE));
        PP512(A[0x3], A[0x2], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M2(0xF));

        PP512(A[0x4], A[0x3], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M2(0x0));
        PP512(A[0x5], A[0x4], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M2(0x1));
        PP512(A[0x6], A[0x5], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M2(0x2));
        PP512(A[0x7], A[0x6], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M2(0x3));
        PP512(A[0x8], A[0x7], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M2(0x4));
        PP512(A[0x9], A[0x8], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M2(0x5));
        PP512(A[0xA], A[0x9], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M2(0x6));
        PP512(A[0xB], A[0xA], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M2(0x7));
        PP512(A[0x0], A[0xB], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M2(0x8));
        PP512(A[0x1], A[0x0], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M2(0x9));
        PP512(A[0x2], A[0x1], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M2(0xA));
        PP512(A[0x3], A[0x2], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M2(0xB));
        PP512(A[0x4], A[0x3], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M2(0xC));
        PP512(A[0x5], A[0x4], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M2(0xD));
        PP512(A[0x6], A[0x5], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M2(0xE));
        PP512(A[0x7], A[0x6], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M2(0xF));

        PP512(A[0x8], A[0x7], B[0x0], B[0xD], B[0x9], B[0x6], C[0x8], M2(0x0));
        PP512(A[0x9], A[0x8], B[0x1], B[0xE], B[0xA], B[0x7], C[0x7], M2(0x1));
        PP512(A[0xA], A[0x9], B[0x2], B[0xF], B[0xB], B[0x8], C[0x6], M2(0x2));
        PP512(A[0xB], A[0xA], B[0x3], B[0x0], B[0xC], B[0x9], C[0x5], M2(0x3));
        PP512(A[0x0], A[0xB], B[0x4], B[0x1], B[0xD], B[0xA], C[0x4], M2(0x4));
        PP512(A[0x1], A[0x0], B[0x5], B[0x2], B[0xE], B[0xB], C[0x3], M2(0x5));
        PP512(A[0x2], A[0x1], B[0x6], B[0x3], B[0xF], B[0xC], C[0x2], M2(0x6));
        PP512(A[0x3], A[0x2], B[0x7], B[0x4], B[0x0], B[0xD], C[0x1], M2(0x7));
        PP512(A[0x4], A[0x3], B[0x8], B[0x5], B[0x1], B[0xE], C[0x0], M2(0x8));
        PP512(A[0x5], A[0x4], B[0x9], B[0x6], B[0x2], B[0xF], C[0xF], M2(0x9));
        PP512(A[0x6], A[0x5], B[0xA], B[0x7], B[0x3], B[0x0], C[0xE], M2(0xA));
        PP512(A[0x7], A[0x6], B[0xB], B[0x8], B[0x4], B[0x1], C[0xD], M2(0xB));
        PP512(A[0x8], A[0x7], B[0xC], B[0x9], B[0x5], B[0x2], C[0xC], M2(0xC));
        PP512(A[0x9], A[0x8], B[0xD], B[0xA], B[0x6], B[0x3], C[0xB], M2(0xD));
        PP512(A[0xA], A[0x9], B[0xE], B[0xB], B[0x7], B[0x4], C[0xA], M2(0xE));
        PP512(A[0xB], A[0xA], B[0xF], B[0xC], B[0x8], B[0x5], C[0x9], M2(0xF));

        A[0xB] = _mm512_add_epi32(A[0xB], C[0x6]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0x5]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0x4]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0x3]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0x2]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x1]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x0]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0xF]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0xE]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0xD]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0xC]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0xB]);
        A[0xB] = _mm512_add_epi32(A[0xB], C[0xA]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0x9]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0x8]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0x7]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0x6]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x5]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x4]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0x3]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0x2]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0x1]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0x0]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0xF]);
        A[0xB] = _mm512_add_epi32(A[0xB], C[0xE]);
        A[0xA] = _mm512_add_epi32(A[0xA], C[0xD]);
        A[0x9] = _mm512_add_epi32(A[0x9], C[0xC]);
        A[0x8] = _mm512_add_epi32(A[0x8], C[0xB]);
        A[0x7] = _mm512_add_epi32(A[0x7], C[0xA]);
        A[0x6] = _mm512_add_epi32(A[0x6], C[0x9]);
        A[0x5] = _mm512_add_epi32(A[0x5], C[0x8]);
        A[0x4] = _mm512_add_epi32(A[0x4], C[0x7]);
        A[0x3] = _mm512_add_epi32(A[0x3], C[0x6]);
        A[0x2] = _mm512_add_epi32(A[0x2], C[0x5]);
        A[0x1] = _mm512_add_epi32(A[0x1], C[0x4]);
        A[0x0] = _mm512_add_epi32(A[0x0], C[0x3]);

        SWAP_AND_SUB512(B[0x0], C[0x0], M2(0x0));
        SWAP_AND_SUB512(B[0x1], C[0x1], M2(0x1));
        SWAP_AND_SUB512(B[0x2], C[0x2], M2(0x2));
        SWAP_AND_SUB512(B[0x3], C[0x3], M2(0x3));
        SWAP_AND_SUB512(B[0x4], C[0x4], M2(0x4));
        SWAP_AND_SUB512(B[0x5], C[0x5], M2(0x5));
        SWAP_AND_SUB512(B[0x6], C[0x6], M2(0x6));
        SWAP_AND_SUB512(B[0x7], C[0x7], M2(0x7));
        SWAP_AND_SUB512(B[0x8], C[0x8], M2(0x8));
        SWAP_AND_SUB512(B[0x9], C[0x9], M2(0x9));
        SWAP_AND_SUB512(B[0xA], C[0xA], M2(0xA));
        SWAP_AND_SUB512(B[0xB], C[0xB], M2(0xB));
        SWAP_AND_SUB512(B[0xC], C[0xC], M2(0xC));
        SWAP_AND_SUB512(B[0xD], C[0xD], M2(0xD));
        SWAP_AND_SUB512(B[0xE], C[0xE], M2(0xE));
        SWAP_AND_SUB512(B[0xF], C[0xF], M2(0xF));

        if (++sc->Wlow == 0) sc->Whigh++;

        if (sc->Wlow-- == 0) sc->Whigh--;
    }

    // download SIMD aligned deadlines
    u32 simd_dst[32];
    _mm512_storeu_si512((__m512i *)&simd_dst[0], C[8]);
    _mm512_storeu_si512((__m512i *)&simd_dst[16], C[9]);

    // unpack SIMD data
    unsigned z;
    for (z = 0; z < 2; z++) {
        unsigned y = z * MSHABAL512_VECTOR_SIZE;
        ((u32 *)dst0)[z] = simd_dst[y + 0];
        ((u32 *)dst1)[z] = simd_dst[y + 1];
        ((u32 *)dst2)[z] = simd_dst[y + 2];
        ((u32 *)dst3)[z] = simd_dst[y + 3];
        ((u32 *)dst4)[z] = simd_dst[y + 4];
        ((u32 *)dst5)[z] = simd_dst[y + 5];
        ((u32 *)dst6)[z] = simd_dst[y + 6];
        ((u32 *)dst7)[z] = simd_dst[y + 7];
        ((u32 *)dst8)[z] = simd_dst[y + 8];
        ((u32 *)dst9)[z] = simd_dst[y + 9];
        ((u32 *)dst10)[z] = simd_dst[y + 10];
        ((u32 *)dst11)[z] = simd_dst[y + 11];
        ((u32 *)dst12)[z] = simd_dst[y + 12];
        ((u32 *)dst13)[z] = simd_dst[y + 13];
        ((u32 *)dst14)[z] = simd_dst[y + 14];
        ((u32 *)dst15)[z] = simd_dst[y + 15];
    }

    // reset Wlow & Whigh
    sc->Wlow = 1;
    sc->Whigh = 0;
}

#ifdef __cplusplus
}
#endif
