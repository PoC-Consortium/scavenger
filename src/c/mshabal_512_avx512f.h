/*
 * A parallel implementation of Shabal, for platforms with AVX512F.
 *
 * This is the header file for an implementation of the Shabal family
 * of hash functions, designed for maximum parallel speed. It processes
 * up to four instances of Shabal in parallel, using the AVX512F unit.
 * Total bandwidth appear to be up to twice that of a plain 32-bit
 * Shabal implementation.
 *
 * A computation uses a mshabal_context structure. That structure is
 * supposed to be allocated and released by the caller, e.g. as a
 * local or global variable, or on the heap. The structure contents
 * are initialized with mshabal_init(). Once the structure has been
 * initialized, data is input as chunks, with the mshabal() functions.
 * Chunks for the four parallel instances are provided simultaneously
 * and must have the same length. It is allowed not to use some of the
 * instances; the corresponding parameters in mshabal() are then NULL.
 * However, using NULL as a chunk for one of the instances effectively
 * deactivates that instance; this cannot be used to "skip" a chunk
 * for one instance.
 *
 * The computation is finalized with mshabal_close(). Some extra message
 * bits (0 to 7) can be input. The outputs of the four parallel instances
 * are written in the provided buffers. There again, NULL can be
 * provided as parameter is the output of one of the instances is not
 * needed.
 *
 * A mshabal_context instance is self-contained and holds no pointer.
 * Thus, it can be cloned (e.g. with memcpy()) or moved (as long as
 * proper alignment is maintained). This implementation uses no state
 * variable beyond the context instance; this, it is thread-safe and
 * reentrant.
 *
 * The Shabal specification defines Shabal with output sizes of 192,
 * 224, 256, 384 and 512 bits. This code accepts all those sizes, as
 * well as any output size which is multiple of 32, between 32 and
 * 512 (inclusive).
 *
 * Parameters are not validated. Thus, undefined behaviour occurs if
 * any of the "shall" or "must" clauses in this documentation is
 * violated.
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

#ifndef MSHABAL_H__
#define MSHABAL_H__

#include <limits.h>

#ifdef __cplusplus
extern "C" {
#endif

/*
 * We need an integer type with width 32-bit or more (preferably, with
 * a width of exactly 32 bits).
 */
#if defined __STDC__ && __STDC_VERSION__ >= 199901L
#include <stdint.h>
#ifdef UINT32_MAX
typedef uint32_t mshabal_u32;
#else
typedef uint_fast32_t mshabal_u32;
#endif
#else
#if ((UINT_MAX >> 11) >> 11) >= 0x3FF
typedef unsigned int mshabal_u32;
#else
typedef unsigned long mshabal_u32;
#endif
#endif

/*
 * The context structure for a Shabal computation. Contents are
 * private. Such a structure should be allocated and released by
 * the caller, in any memory area.
 */

#define MSHABAL512_FACTOR 4

/*
 * The context structure for a Shabal computation. Contents are
 * private. Such a structure should be allocated and released by
 * the caller, in any memory area.
 */
typedef struct {
    unsigned char buf0[64];
    unsigned char buf1[64];
    unsigned char buf2[64];
    unsigned char buf3[64];
    unsigned char buf4[64];
    unsigned char buf5[64];
    unsigned char buf6[64];
    unsigned char buf7[64];
    unsigned char buf8[64];
    unsigned char buf9[64];
    unsigned char buf10[64];
    unsigned char buf11[64];
    unsigned char buf12[64];
    unsigned char buf13[64];
    unsigned char buf14[64];
    unsigned char buf15[64];
    unsigned char* xbuf0;
    unsigned char* xbuf1;
    unsigned char* xbuf2;
    unsigned char* xbuf3;
    unsigned char* xbuf4;
    unsigned char* xbuf5;
    unsigned char* xbuf6;
    unsigned char* xbuf7;
    unsigned char* xbuf8;
    unsigned char* xbuf9;
    unsigned char* xbuf10;
    unsigned char* xbuf11;
    unsigned char* xbuf12;
    unsigned char* xbuf13;
    unsigned char* xbuf14;
    unsigned char* xbuf15;
    size_t ptr;
    mshabal_u32 state[(12 + 16 + 16) * 4 * MSHABAL512_FACTOR];
    mshabal_u32 Whigh, Wlow;
    unsigned out_size;
} mshabal512_context;

#pragma pack(1)
typedef struct {
    mshabal_u32 state[(12 + 16 + 16) * 4 * MSHABAL512_FACTOR];
    mshabal_u32 Whigh, Wlow;
    unsigned out_size;
} mshabal512_context_fast;
#pragma pack()

/*
 * Initialize a context structure. The output size must be a multiple
 * of 32, between 32 and 512 (inclusive). The output size is expressed
 * in bits.
 */

void simd512_mshabal_init(mshabal512_context *sc, unsigned out_size);

/*
 * Process some more data bytes; four chunks of data, pointed to by
 * data0, data1, data2 and data3, are processed. The four chunks have
 * the same length of "len" bytes. For efficiency, it is best if data is
 * processed by medium-sized chunks, e.g. a few kilobytes at a time.
 *
 * The "len" data bytes shall all be accessible. If "len" is zero, this
 * this function does nothing and ignores the data* arguments.
 * Otherwise, if one of the data* argument is NULL, then the
 * corresponding instance is deactivated (the final value obtained from
 * that instance is undefined).
 */

void simd512_mshabal(mshabal512_context *sc, void *data0, void *data1, void *data2, void *data3,
                     void *data4, void *data5, void *data6, void *data7, void *data8, void *data9,
                     void *data10, void *data11, void *data12, void *data13, void *data14,
                     void *data15, size_t len);
/*
 * Terminate the Shabal computation incarnated by the provided context
 * structure. "n" shall be a value between 0 and 7 (inclusive): this is
 * the number of extra bits to extract from ub0, ub1, ub2 and ub3, and
 * append at the end of the input message for each of the four parallel
 * instances. Bits in "ub*" are taken in big-endian format: first bit is
 * the one of numerical value 128, second bit has numerical value 64,
 * and so on. Other bits in "ub*" are ignored. For most applications,
 * input messages will consist in sequence of bytes, and the "ub*" and
 * "n" parameters will be zero.
 *
 * The Shabal output for each of the parallel instances is written out
 * in the areas pointed to by, respectively, dst0, dst1, dst2 and dst3.
 * These areas shall be wide enough to accomodate the result (result
 * size was specified as parameter to mshabal_init()). It is acceptable
 * to use NULL for any of those pointers, if the result from the
 * corresponding instance is not needed.
 *
 * After this call, the context structure is invalid. The caller shall
 * release it, or reinitialize it with mshabal_init(). The mshabal_close()
 * function does NOT imply a hidden call to mshabal_init().
 */

void simd512_mshabal_close(mshabal512_context *sc, unsigned ub0, unsigned ub1, unsigned ub2,
                           unsigned ub3, unsigned ub4, unsigned ub5, unsigned ub6, unsigned ub7,
                           unsigned ub8, unsigned ub9, unsigned ub10, unsigned ub11, unsigned ub12,
                           unsigned ub13, unsigned ub14, unsigned ub15, unsigned n, void *dst0,
                           void *dst1, void *dst2, void *dst3, void *dst4, void *dst5, void *dst6,
                           void *dst7, void *dst8, void *dst9, void *dst10, void *dst11,
                           void *dst12, void *dst13, void *dst14, void *dst15);

/*
 * Combined open and close routines
 */

void simd512_mshabal_openclose_fast(mshabal512_context_fast *sc, void *u1, void *u2, void *dst0,
                                    void *dst1, void *dst2, void *dst3, void *dst4, void *dst5,
                                    void *dst6, void *dst7, void *dst8, void *dst9, void *dst10,
                                    void *dst11, void *dst12, void *dst13, void *dst14,
                                    void *dst15);
                                    
#ifdef __cplusplus
}
#endif

#endif
