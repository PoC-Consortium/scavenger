/* $Id: shabal.c 175 2010-05-07 16:03:20Z tp $ */
/*
 * Shabal implementation.
 *
 * ==========================(LICENSE BEGIN)============================
 *
 * Copyright (c) 2007-2010  Projet RNRT SAPHIR
 *
 * Permission is hereby granted, free of charge, to any person obtaining
 * a copy of this software and associated documentation files (the
 * "Software"), to deal in the Software without restriction, including
 * without limitation the rights to use, copy, modify, merge, publish,
 * distribute, sublicense, and/or sell copies of the Software, and to
 * permit persons to whom the Software is furnished to do so, subject to
 * the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
 * IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
 * CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
 * TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
 * SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 *
 * ===========================(LICENSE END)=============================
 *
 * @author   Thomas Pornin <thomas.pornin@cryptolog.com>
 */

#include <stddef.h>
#include <string.h>

#include "sph_shabal.h"

#ifdef _MSC_VER
#pragma warning(disable : 4146)
#endif

/*
 * Part of this code was automatically generated (the part between
 * the "BEGIN" and "END" markers).
 */

#define sM 16

//#define C32   SPH_C32
//#define T32   SPH_T32

//#define O1   13
//#define O2    9
//#define O3    6

/*
 * We copy the state into local variables, so that the compiler knows
 * that it can optimize them at will.
 */

/* BEGIN -- automatically generated code. */

#define DECL_STATE                                                          \
    sph_u32 A00, A01, A02, A03, A04, A05, A06, A07, A08, A09, A0A, A0B;     \
    sph_u32 B0, B1, B2, B3, B4, B5, B6, B7, B8, B9, BA, BB, BC, BD, BE, BF; \
    sph_u32 C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, CA, CB, CC, CD, CE, CF; \
    sph_u32 M0, M1, M2, M3, M4, M5, M6, M7, M8, M9, MA, MB, MC, MD, ME, MF; \
    sph_u32 Wlow, Whigh;

#define READ_STATE(state)       \
    do {                        \
        A00 = (state)->A[0];    \
        A01 = (state)->A[1];    \
        A02 = (state)->A[2];    \
        A03 = (state)->A[3];    \
        A04 = (state)->A[4];    \
        A05 = (state)->A[5];    \
        A06 = (state)->A[6];    \
        A07 = (state)->A[7];    \
        A08 = (state)->A[8];    \
        A09 = (state)->A[9];    \
        A0A = (state)->A[10];   \
        A0B = (state)->A[11];   \
        B0 = (state)->B[0];     \
        B1 = (state)->B[1];     \
        B2 = (state)->B[2];     \
        B3 = (state)->B[3];     \
        B4 = (state)->B[4];     \
        B5 = (state)->B[5];     \
        B6 = (state)->B[6];     \
        B7 = (state)->B[7];     \
        B8 = (state)->B[8];     \
        B9 = (state)->B[9];     \
        BA = (state)->B[10];    \
        BB = (state)->B[11];    \
        BC = (state)->B[12];    \
        BD = (state)->B[13];    \
        BE = (state)->B[14];    \
        BF = (state)->B[15];    \
        C0 = (state)->C[0];     \
        C1 = (state)->C[1];     \
        C2 = (state)->C[2];     \
        C3 = (state)->C[3];     \
        C4 = (state)->C[4];     \
        C5 = (state)->C[5];     \
        C6 = (state)->C[6];     \
        C7 = (state)->C[7];     \
        C8 = (state)->C[8];     \
        C9 = (state)->C[9];     \
        CA = (state)->C[10];    \
        CB = (state)->C[11];    \
        CC = (state)->C[12];    \
        CD = (state)->C[13];    \
        CE = (state)->C[14];    \
        CF = (state)->C[15];    \
        Wlow = (state)->Wlow;   \
        Whigh = (state)->Whigh; \
    } while (0)

#define WRITE_STATE(state)      \
    do {                        \
        (state)->A[0] = A00;    \
        (state)->A[1] = A01;    \
        (state)->A[2] = A02;    \
        (state)->A[3] = A03;    \
        (state)->A[4] = A04;    \
        (state)->A[5] = A05;    \
        (state)->A[6] = A06;    \
        (state)->A[7] = A07;    \
        (state)->A[8] = A08;    \
        (state)->A[9] = A09;    \
        (state)->A[10] = A0A;   \
        (state)->A[11] = A0B;   \
        (state)->B[0] = B0;     \
        (state)->B[1] = B1;     \
        (state)->B[2] = B2;     \
        (state)->B[3] = B3;     \
        (state)->B[4] = B4;     \
        (state)->B[5] = B5;     \
        (state)->B[6] = B6;     \
        (state)->B[7] = B7;     \
        (state)->B[8] = B8;     \
        (state)->B[9] = B9;     \
        (state)->B[10] = BA;    \
        (state)->B[11] = BB;    \
        (state)->B[12] = BC;    \
        (state)->B[13] = BD;    \
        (state)->B[14] = BE;    \
        (state)->B[15] = BF;    \
        (state)->C[0] = C0;     \
        (state)->C[1] = C1;     \
        (state)->C[2] = C2;     \
        (state)->C[3] = C3;     \
        (state)->C[4] = C4;     \
        (state)->C[5] = C5;     \
        (state)->C[6] = C6;     \
        (state)->C[7] = C7;     \
        (state)->C[8] = C8;     \
        (state)->C[9] = C9;     \
        (state)->C[10] = CA;    \
        (state)->C[11] = CB;    \
        (state)->C[12] = CC;    \
        (state)->C[13] = CD;    \
        (state)->C[14] = CE;    \
        (state)->C[15] = CF;    \
        (state)->Wlow = Wlow;   \
        (state)->Whigh = Whigh; \
    } while (0)

#define DECODE_BLOCK                        \
    do {                                    \
        M0 = sph_dec32le_aligned(buf + 0);  \
        M1 = sph_dec32le_aligned(buf + 4);  \
        M2 = sph_dec32le_aligned(buf + 8);  \
        M3 = sph_dec32le_aligned(buf + 12); \
        M4 = sph_dec32le_aligned(buf + 16); \
        M5 = sph_dec32le_aligned(buf + 20); \
        M6 = sph_dec32le_aligned(buf + 24); \
        M7 = sph_dec32le_aligned(buf + 28); \
        M8 = sph_dec32le_aligned(buf + 32); \
        M9 = sph_dec32le_aligned(buf + 36); \
        MA = sph_dec32le_aligned(buf + 40); \
        MB = sph_dec32le_aligned(buf + 44); \
        MC = sph_dec32le_aligned(buf + 48); \
        MD = sph_dec32le_aligned(buf + 52); \
        ME = sph_dec32le_aligned(buf + 56); \
        MF = sph_dec32le_aligned(buf + 60); \
    } while (0)

#define INPUT_BLOCK_ADD        \
    do {                       \
        B0 = SPH_T32(B0 + M0); \
        B1 = SPH_T32(B1 + M1); \
        B2 = SPH_T32(B2 + M2); \
        B3 = SPH_T32(B3 + M3); \
        B4 = SPH_T32(B4 + M4); \
        B5 = SPH_T32(B5 + M5); \
        B6 = SPH_T32(B6 + M6); \
        B7 = SPH_T32(B7 + M7); \
        B8 = SPH_T32(B8 + M8); \
        B9 = SPH_T32(B9 + M9); \
        BA = SPH_T32(BA + MA); \
        BB = SPH_T32(BB + MB); \
        BC = SPH_T32(BC + MC); \
        BD = SPH_T32(BD + MD); \
        BE = SPH_T32(BE + ME); \
        BF = SPH_T32(BF + MF); \
    } while (0)

#define INPUT_BLOCK_SUB        \
    do {                       \
        C0 = SPH_T32(C0 - M0); \
        C1 = SPH_T32(C1 - M1); \
        C2 = SPH_T32(C2 - M2); \
        C3 = SPH_T32(C3 - M3); \
        C4 = SPH_T32(C4 - M4); \
        C5 = SPH_T32(C5 - M5); \
        C6 = SPH_T32(C6 - M6); \
        C7 = SPH_T32(C7 - M7); \
        C8 = SPH_T32(C8 - M8); \
        C9 = SPH_T32(C9 - M9); \
        CA = SPH_T32(CA - MA); \
        CB = SPH_T32(CB - MB); \
        CC = SPH_T32(CC - MC); \
        CD = SPH_T32(CD - MD); \
        CE = SPH_T32(CE - ME); \
        CF = SPH_T32(CF - MF); \
    } while (0)

#define XOR_W         \
    do {              \
        A00 ^= Wlow;  \
        A01 ^= Whigh; \
    } while (0)

#define SWAP(v1, v2)        \
    do {                    \
        sph_u32 tmp = (v1); \
        (v1) = (v2);        \
        (v2) = tmp;         \
    } while (0)

#define SWAP_BC       \
    do {              \
        SWAP(B0, C0); \
        SWAP(B1, C1); \
        SWAP(B2, C2); \
        SWAP(B3, C3); \
        SWAP(B4, C4); \
        SWAP(B5, C5); \
        SWAP(B6, C6); \
        SWAP(B7, C7); \
        SWAP(B8, C8); \
        SWAP(B9, C9); \
        SWAP(BA, CA); \
        SWAP(BB, CB); \
        SWAP(BC, CC); \
        SWAP(BD, CD); \
        SWAP(BE, CE); \
        SWAP(BF, CF); \
    } while (0)

#define PERM_ELT(xa0, xa1, xb0, xb1, xb2, xb3, xc, xm)                                             \
    do {                                                                                           \
        xa0 = SPH_T32((xa0 ^ (((xa1 << 15) | (xa1 >> 17)) * 5U) ^ xc) * 3U) ^ xb1 ^ (xb2 & ~xb3) ^ \
              xm;                                                                                  \
        xb0 = SPH_T32(~(((xb0 << 1) | (xb0 >> 31)) ^ xa0));                                        \
    } while (0)

#define PERM_STEP_0                                 \
    do {                                            \
        PERM_ELT(A00, A0B, B0, BD, B9, B6, C8, M0); \
        PERM_ELT(A01, A00, B1, BE, BA, B7, C7, M1); \
        PERM_ELT(A02, A01, B2, BF, BB, B8, C6, M2); \
        PERM_ELT(A03, A02, B3, B0, BC, B9, C5, M3); \
        PERM_ELT(A04, A03, B4, B1, BD, BA, C4, M4); \
        PERM_ELT(A05, A04, B5, B2, BE, BB, C3, M5); \
        PERM_ELT(A06, A05, B6, B3, BF, BC, C2, M6); \
        PERM_ELT(A07, A06, B7, B4, B0, BD, C1, M7); \
        PERM_ELT(A08, A07, B8, B5, B1, BE, C0, M8); \
        PERM_ELT(A09, A08, B9, B6, B2, BF, CF, M9); \
        PERM_ELT(A0A, A09, BA, B7, B3, B0, CE, MA); \
        PERM_ELT(A0B, A0A, BB, B8, B4, B1, CD, MB); \
        PERM_ELT(A00, A0B, BC, B9, B5, B2, CC, MC); \
        PERM_ELT(A01, A00, BD, BA, B6, B3, CB, MD); \
        PERM_ELT(A02, A01, BE, BB, B7, B4, CA, ME); \
        PERM_ELT(A03, A02, BF, BC, B8, B5, C9, MF); \
    } while (0)

#define PERM_STEP_1                                 \
    do {                                            \
        PERM_ELT(A04, A03, B0, BD, B9, B6, C8, M0); \
        PERM_ELT(A05, A04, B1, BE, BA, B7, C7, M1); \
        PERM_ELT(A06, A05, B2, BF, BB, B8, C6, M2); \
        PERM_ELT(A07, A06, B3, B0, BC, B9, C5, M3); \
        PERM_ELT(A08, A07, B4, B1, BD, BA, C4, M4); \
        PERM_ELT(A09, A08, B5, B2, BE, BB, C3, M5); \
        PERM_ELT(A0A, A09, B6, B3, BF, BC, C2, M6); \
        PERM_ELT(A0B, A0A, B7, B4, B0, BD, C1, M7); \
        PERM_ELT(A00, A0B, B8, B5, B1, BE, C0, M8); \
        PERM_ELT(A01, A00, B9, B6, B2, BF, CF, M9); \
        PERM_ELT(A02, A01, BA, B7, B3, B0, CE, MA); \
        PERM_ELT(A03, A02, BB, B8, B4, B1, CD, MB); \
        PERM_ELT(A04, A03, BC, B9, B5, B2, CC, MC); \
        PERM_ELT(A05, A04, BD, BA, B6, B3, CB, MD); \
        PERM_ELT(A06, A05, BE, BB, B7, B4, CA, ME); \
        PERM_ELT(A07, A06, BF, BC, B8, B5, C9, MF); \
    } while (0)

#define PERM_STEP_2                                 \
    do {                                            \
        PERM_ELT(A08, A07, B0, BD, B9, B6, C8, M0); \
        PERM_ELT(A09, A08, B1, BE, BA, B7, C7, M1); \
        PERM_ELT(A0A, A09, B2, BF, BB, B8, C6, M2); \
        PERM_ELT(A0B, A0A, B3, B0, BC, B9, C5, M3); \
        PERM_ELT(A00, A0B, B4, B1, BD, BA, C4, M4); \
        PERM_ELT(A01, A00, B5, B2, BE, BB, C3, M5); \
        PERM_ELT(A02, A01, B6, B3, BF, BC, C2, M6); \
        PERM_ELT(A03, A02, B7, B4, B0, BD, C1, M7); \
        PERM_ELT(A04, A03, B8, B5, B1, BE, C0, M8); \
        PERM_ELT(A05, A04, B9, B6, B2, BF, CF, M9); \
        PERM_ELT(A06, A05, BA, B7, B3, B0, CE, MA); \
        PERM_ELT(A07, A06, BB, B8, B4, B1, CD, MB); \
        PERM_ELT(A08, A07, BC, B9, B5, B2, CC, MC); \
        PERM_ELT(A09, A08, BD, BA, B6, B3, CB, MD); \
        PERM_ELT(A0A, A09, BE, BB, B7, B4, CA, ME); \
        PERM_ELT(A0B, A0A, BF, BC, B8, B5, C9, MF); \
    } while (0)

#define APPLY_P                              \
    do {                                     \
        B0 = SPH_T32(B0 << 17) | (B0 >> 15); \
        B1 = SPH_T32(B1 << 17) | (B1 >> 15); \
        B2 = SPH_T32(B2 << 17) | (B2 >> 15); \
        B3 = SPH_T32(B3 << 17) | (B3 >> 15); \
        B4 = SPH_T32(B4 << 17) | (B4 >> 15); \
        B5 = SPH_T32(B5 << 17) | (B5 >> 15); \
        B6 = SPH_T32(B6 << 17) | (B6 >> 15); \
        B7 = SPH_T32(B7 << 17) | (B7 >> 15); \
        B8 = SPH_T32(B8 << 17) | (B8 >> 15); \
        B9 = SPH_T32(B9 << 17) | (B9 >> 15); \
        BA = SPH_T32(BA << 17) | (BA >> 15); \
        BB = SPH_T32(BB << 17) | (BB >> 15); \
        BC = SPH_T32(BC << 17) | (BC >> 15); \
        BD = SPH_T32(BD << 17) | (BD >> 15); \
        BE = SPH_T32(BE << 17) | (BE >> 15); \
        BF = SPH_T32(BF << 17) | (BF >> 15); \
        PERM_STEP_0;                         \
        PERM_STEP_1;                         \
        PERM_STEP_2;                         \
        A0B = SPH_T32(A0B + C6);             \
        A0A = SPH_T32(A0A + C5);             \
        A09 = SPH_T32(A09 + C4);             \
        A08 = SPH_T32(A08 + C3);             \
        A07 = SPH_T32(A07 + C2);             \
        A06 = SPH_T32(A06 + C1);             \
        A05 = SPH_T32(A05 + C0);             \
        A04 = SPH_T32(A04 + CF);             \
        A03 = SPH_T32(A03 + CE);             \
        A02 = SPH_T32(A02 + CD);             \
        A01 = SPH_T32(A01 + CC);             \
        A00 = SPH_T32(A00 + CB);             \
        A0B = SPH_T32(A0B + CA);             \
        A0A = SPH_T32(A0A + C9);             \
        A09 = SPH_T32(A09 + C8);             \
        A08 = SPH_T32(A08 + C7);             \
        A07 = SPH_T32(A07 + C6);             \
        A06 = SPH_T32(A06 + C5);             \
        A05 = SPH_T32(A05 + C4);             \
        A04 = SPH_T32(A04 + C3);             \
        A03 = SPH_T32(A03 + C2);             \
        A02 = SPH_T32(A02 + C1);             \
        A01 = SPH_T32(A01 + C0);             \
        A00 = SPH_T32(A00 + CF);             \
        A0B = SPH_T32(A0B + CE);             \
        A0A = SPH_T32(A0A + CD);             \
        A09 = SPH_T32(A09 + CC);             \
        A08 = SPH_T32(A08 + CB);             \
        A07 = SPH_T32(A07 + CA);             \
        A06 = SPH_T32(A06 + C9);             \
        A05 = SPH_T32(A05 + C8);             \
        A04 = SPH_T32(A04 + C7);             \
        A03 = SPH_T32(A03 + C6);             \
        A02 = SPH_T32(A02 + C5);             \
        A01 = SPH_T32(A01 + C4);             \
        A00 = SPH_T32(A00 + C3);             \
    } while (0)

#define INCR_W                                                           \
    do {                                                                 \
        if ((Wlow = SPH_T32(Wlow + 1)) == 0) Whigh = SPH_T32(Whigh + 1); \
    } while (0)

static const sph_u32 A_init_256[] = {SPH_C32(0x52F84552), SPH_C32(0xE54B7999), SPH_C32(0x2D8EE3EC),
                                     SPH_C32(0xB9645191), SPH_C32(0xE0078B86), SPH_C32(0xBB7C44C9),
                                     SPH_C32(0xD2B5C1CA), SPH_C32(0xB0D2EB8C), SPH_C32(0x14CE5A45),
                                     SPH_C32(0x22AF50DC), SPH_C32(0xEFFDBC6B), SPH_C32(0xEB21B74A)};

static const sph_u32 B_init_256[] = {
    SPH_C32(0xB555C6EE), SPH_C32(0x3E710596), SPH_C32(0xA72A652F), SPH_C32(0x9301515F),
    SPH_C32(0xDA28C1FA), SPH_C32(0x696FD868), SPH_C32(0x9CB6BF72), SPH_C32(0x0AFE4002),
    SPH_C32(0xA6E03615), SPH_C32(0x5138C1D4), SPH_C32(0xBE216306), SPH_C32(0xB38B8890),
    SPH_C32(0x3EA8B96B), SPH_C32(0x3299ACE4), SPH_C32(0x30924DD4), SPH_C32(0x55CB34A5)};

static const sph_u32 C_init_256[] = {
    SPH_C32(0xB405F031), SPH_C32(0xC4233EBA), SPH_C32(0xB3733979), SPH_C32(0xC0DD9D55),
    SPH_C32(0xC51C28AE), SPH_C32(0xA327B8E1), SPH_C32(0x56C56167), SPH_C32(0xED614433),
    SPH_C32(0x88B59D60), SPH_C32(0x60E2CEBA), SPH_C32(0x758B4B8B), SPH_C32(0x83E82A7F),
    SPH_C32(0xBC968828), SPH_C32(0xE6E00BF7), SPH_C32(0xBA839E55), SPH_C32(0x9B491C60)};

/* END -- automatically generated code. */

void sph_shabal256_init(sph_shabal_context* cc) {
    /*
     * We have precomputed initial states for all the supported
     * output bit lengths.
     */
    // const sph_u32 *A_init, *B_init, *C_init;
    // sph_shabal_context *sc;

    // A_init = A_init_256;
    // B_init = B_init_256;
    // C_init = C_init_256;

    // sc = (sph_shabal_context *) cc;
    memcpy(cc->A, A_init_256, sizeof cc->A);
    memcpy(cc->B, B_init_256, sizeof cc->B);
    memcpy(cc->C, C_init_256, sizeof cc->C);
    cc->Wlow = 1;
    cc->Whigh = 0;
    cc->ptr = 0;
}

void sph_shabal256(void* cc, const unsigned char* data, size_t len) {
    sph_shabal_context* sc;
    unsigned char* buf;
    size_t ptr;
    DECL_STATE

    sc = (sph_shabal_context*)cc;
    buf = sc->buf;
    ptr = sc->ptr;

    /*
     * We do not want to copy the state to local variables if the
     * amount of data is less than what is needed to complete the
     * current block. Note that it is anyway suboptimal to call
     * this method many times for small chunks of data.
     */
    if (len < (sizeof sc->buf) - ptr) {
        memcpy(buf + ptr, data, len);
        ptr += len;
        sc->ptr = ptr;
        return;
    }

    READ_STATE(sc);
    while (len > 0) {
        size_t clen;

        clen = (sizeof sc->buf) - ptr;
        if (clen > len) clen = len;
        memcpy(buf + ptr, data, clen);
        ptr += clen;
        data += clen;
        len -= clen;
        if (ptr == sizeof sc->buf) {
            DECODE_BLOCK;
            INPUT_BLOCK_ADD;
            XOR_W;
            APPLY_P;
            INPUT_BLOCK_SUB;
            SWAP_BC;
            INCR_W;
            ptr = 0;
        }
    }
    WRITE_STATE(sc);
    sc->ptr = ptr;
}

static void shabal_close(void* cc, unsigned ub, unsigned n, void* dst, unsigned size_words) {
    sph_shabal_context* sc;
    unsigned char* buf;
    size_t ptr;
    unsigned z;
    union {
        unsigned char tmp_out[64];
        sph_u32 dummy;
    } u;
    size_t out_len;
    DECL_STATE

    sc = (sph_shabal_context*)cc;
    buf = sc->buf;
    ptr = sc->ptr;
    z = 0x80 >> n;
    buf[ptr] = ((ub & -z) | z) & 0xFF;
    memset(buf + ptr + 1, 0, (sizeof sc->buf) - (ptr + 1));
    READ_STATE(sc);
    DECODE_BLOCK;
    INPUT_BLOCK_ADD;
    XOR_W;
    APPLY_P;
    //#pragma loop(hint_parallel(3))
    for (int i = 0; i < 3; i++) {
        SWAP_BC;
        XOR_W;
        APPLY_P;
    }

    /*
     * We just use our local variables; no need to go through
     * the state structure. In order to share some code, we
     * emit the relevant words into a temporary buffer, which
     * we finally copy into the destination array.
     */

    sph_enc32le_aligned(u.tmp_out + 32, B8);

    sph_enc32le_aligned(u.tmp_out + 36, B9);

    sph_enc32le_aligned(u.tmp_out + 40, BA);
    sph_enc32le_aligned(u.tmp_out + 44, BB);
    sph_enc32le_aligned(u.tmp_out + 48, BC);
    sph_enc32le_aligned(u.tmp_out + 52, BD);
    sph_enc32le_aligned(u.tmp_out + 56, BE);
    sph_enc32le_aligned(u.tmp_out + 60, BF);

    out_len = size_words << 2;
    memcpy(dst, u.tmp_out + (sizeof u.tmp_out) - out_len, out_len);
    // sph_shabal256_init(sc, size_words << 5);
}

/* see sph_shabal.h */
void sph_shabal256_close(void* cc, void* dst) { shabal_close(cc, 0, 0, dst, 8); }

/* see sph_shabal.h */
void sph_shabal256_addbits_and_close(void* cc, unsigned ub, unsigned n, void* dst) {
    shabal_close(cc, ub, n, dst, 8);
}

// Shabal routines optimized for plotting and hashing
void sph_shabal_hash_fast(void *message, void *termination, void* dst, unsigned num) {
    sph_u32
        A00 = A_init_256[0], A01 = A_init_256[1], A02 = A_init_256[2], A03 = A_init_256[3],
        A04 = A_init_256[4], A05 = A_init_256[5], A06 = A_init_256[6], A07 = A_init_256[7],
        A08 = A_init_256[8], A09 = A_init_256[9], A0A = A_init_256[10], A0B = A_init_256[11];
    sph_u32
        B0 = B_init_256[0], B1 = B_init_256[1], B2 = B_init_256[2], B3 = B_init_256[3],
        B4 = B_init_256[4], B5 = B_init_256[5], B6 = B_init_256[6], B7 = B_init_256[7],
        B8 = B_init_256[8], B9 = B_init_256[9], BA = B_init_256[10], BB = B_init_256[11],
        BC = B_init_256[12], BD = B_init_256[13], BE = B_init_256[14], BF = B_init_256[15];
    sph_u32
        C0 = C_init_256[0], C1 = C_init_256[1], C2 = C_init_256[2], C3 = C_init_256[3],
        C4 = C_init_256[4], C5 = C_init_256[5], C6 = C_init_256[6], C7 = C_init_256[7],
        C8 = C_init_256[8], C9 = C_init_256[9], CA = C_init_256[10], CB = C_init_256[11],
        CC = C_init_256[12], CD = C_init_256[13], CE = C_init_256[14], CF = C_init_256[15];
    sph_u32 M0, M1, M2, M3, M4, M5, M6, M7, M8, M9, MA, MB, MC, MD, ME, MF;
    sph_u32 Wlow = 1, Whigh = 0;
         
    while (num-- > 0) {
	    M0 = ((unsigned int *)message)[0];
	    M1 = ((unsigned int *)message)[1];
	    M2 = ((unsigned int *)message)[2];
	    M3 = ((unsigned int *)message)[3];
	    M4 = ((unsigned int *)message)[4];
	    M5 = ((unsigned int *)message)[5];
	    M6 = ((unsigned int *)message)[6];
	    M7 = ((unsigned int *)message)[7];
	    M8 = ((unsigned int *)message)[8];
	    M9 = ((unsigned int *)message)[9];
	    MA = ((unsigned int *)message)[10];
	    MB = ((unsigned int *)message)[11];
	    MC = ((unsigned int *)message)[12];
	    MD = ((unsigned int *)message)[13];
	    ME = ((unsigned int *)message)[14];
    	MF = ((unsigned int *)message)[15];

        INPUT_BLOCK_ADD;
        XOR_W;
        APPLY_P;
        INPUT_BLOCK_SUB;
        SWAP_BC;
        INCR_W;

        message = (unsigned int *)message + 16;
    }

	    M0 = ((unsigned int *)termination)[0];
	    M1 = ((unsigned int *)termination)[1];
	    M2 = ((unsigned int *)termination)[2];
	    M3 = ((unsigned int *)termination)[3];
	    M4 = ((unsigned int *)termination)[4];
	    M5 = ((unsigned int *)termination)[5];
	    M6 = ((unsigned int *)termination)[6];
	    M7 = ((unsigned int *)termination)[7];
	    M8 = ((unsigned int *)termination)[8];
	    M9 = ((unsigned int *)termination)[9];
	    MA = ((unsigned int *)termination)[10];
	    MB = ((unsigned int *)termination)[11];
	    MC = ((unsigned int *)termination)[12];
	    MD = ((unsigned int *)termination)[13];
	    ME = ((unsigned int *)termination)[14];
    	MF = ((unsigned int *)termination)[15];

    INPUT_BLOCK_ADD;
    XOR_W;
    APPLY_P;

    for (int i = 0; i < 3; i++) {
        SWAP_BC;
        XOR_W;
        APPLY_P;
    }

    sph_enc32le_aligned((sph_u32 *)dst, B8);
    sph_enc32le_aligned((sph_u32 *)dst + 1, B9);
    sph_enc32le_aligned((sph_u32 *)dst + 2, BA);
    sph_enc32le_aligned((sph_u32 *)dst + 3, BB);
    sph_enc32le_aligned((sph_u32 *)dst + 4, BC);
    sph_enc32le_aligned((sph_u32 *)dst + 5, BD);
    sph_enc32le_aligned((sph_u32 *)dst + 6, BE);
    sph_enc32le_aligned((sph_u32 *)dst + 7, BF);    
}

// Shabal routines optimized for mining
void sph_shabal_deadline_fast(void *scoop_data, void *gen_sig, void *dst) {
    sph_u32
        A00 = A_init_256[0], A01 = A_init_256[1], A02 = A_init_256[2], A03 = A_init_256[3],
        A04 = A_init_256[4], A05 = A_init_256[5], A06 = A_init_256[6], A07 = A_init_256[7],
        A08 = A_init_256[8], A09 = A_init_256[9], A0A = A_init_256[10], A0B = A_init_256[11];
    sph_u32
        B0 = B_init_256[0], B1 = B_init_256[1], B2 = B_init_256[2], B3 = B_init_256[3],
        B4 = B_init_256[4], B5 = B_init_256[5], B6 = B_init_256[6], B7 = B_init_256[7],
        B8 = B_init_256[8], B9 = B_init_256[9], BA = B_init_256[10], BB = B_init_256[11],
        BC = B_init_256[12], BD = B_init_256[13], BE = B_init_256[14], BF = B_init_256[15];
    sph_u32
        C0 = C_init_256[0], C1 = C_init_256[1], C2 = C_init_256[2], C3 = C_init_256[3],
        C4 = C_init_256[4], C5 = C_init_256[5], C6 = C_init_256[6], C7 = C_init_256[7],
        C8 = C_init_256[8], C9 = C_init_256[9], CA = C_init_256[10], CB = C_init_256[11],
        CC = C_init_256[12], CD = C_init_256[13], CE = C_init_256[14], CF = C_init_256[15];
    sph_u32 M0, M1, M2, M3, M4, M5, M6, M7, M8, M9, MA, MB, MC, MD, ME, MF;
    sph_u32 Wlow = 1, Whigh = 0;
         
	M0 = ((unsigned int *)gen_sig)[0];
	M1 = ((unsigned int *)gen_sig)[1];
	M2 = ((unsigned int *)gen_sig)[2];
	M3 = ((unsigned int *)gen_sig)[3];
	M4 = ((unsigned int *)gen_sig)[4];
	M5 = ((unsigned int *)gen_sig)[5];
	M6 = ((unsigned int *)gen_sig)[6];
	M7 = ((unsigned int *)gen_sig)[7];
	M8 = ((unsigned int *)scoop_data)[0];
	M9 = ((unsigned int *)scoop_data)[1];
	MA = ((unsigned int *)scoop_data)[2];
	MB = ((unsigned int *)scoop_data)[3];
	MC = ((unsigned int *)scoop_data)[4];
	MD = ((unsigned int *)scoop_data)[5];
	ME = ((unsigned int *)scoop_data)[6];
	MF = ((unsigned int *)scoop_data)[7];

    INPUT_BLOCK_ADD;
    XOR_W;
    APPLY_P;
    INPUT_BLOCK_SUB;
    SWAP_BC;
    INCR_W;

	M0 = ((unsigned int *)scoop_data)[8];
	M1 = ((unsigned int *)scoop_data)[9];
	M2 = ((unsigned int *)scoop_data)[10];
	M3 = ((unsigned int *)scoop_data)[11];
	M4 = ((unsigned int *)scoop_data)[12];
	M5 = ((unsigned int *)scoop_data)[13];
	M6 = ((unsigned int *)scoop_data)[14];
	M7 = ((unsigned int *)scoop_data)[15];
	M8 = 0x80;
	M9 = MA = MB = MC = MD = ME = MF = 0;

    INPUT_BLOCK_ADD;
    XOR_W;
    APPLY_P;

    for (int i = 0; i < 3; i++) {
        SWAP_BC;
        XOR_W;
        APPLY_P;
    }

    sph_enc32le_aligned((sph_u32 *)dst, B8);
    sph_enc32le_aligned((sph_u32 *)dst + 1, B9);    
}