/*
 * A parallel implementation of Shabal, for platforms with SSE2.
 *
 * This is the header file for an implementation of the Shabal family
 * of hash functions, designed for maximum parallel speed. It processes
 * up to four instances of Shabal in parallel, using the SSE2 unit.
 * Total bandwidth appear to be up to twice that of a plain 32-bit
 * Shabal implementation.
 *
 * A computation uses a mshabal256_context structure. That structure is
 * supposed to be allocated and released by the caller, e.g. as a
 * local or global variable, or on the heap. The structure contents
 * are initialized with mshabal256_init(). Once the structure has been
 * initialized, data is input as chunks, with the mshabal256() functions.
 * Chunks for the four parallel instances are provided simultaneously
 * and must have the same length. It is allowed not to use some of the
 * instances; the corresponding parameters in mshabal256() are then NULL.
 * However, using NULL as a chunk for one of the instances effectively
 * deactivates that instance; this cannot be used to "skip" a chunk
 * for one instance.
 *
 * The computation is finalized with mshabal256_close(). Some extra message
 * bits (0 to 7) can be input. The outputs of the four parallel instances
 * are written in the provided buffers. There again, NULL can be
 * provided as parameter is the output of one of the instances is not
 * needed.
 *
 * A mshabal256_context instance is self-contained and holds no pointer.
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

#ifndef MSHABAL256_H__
#define MSHABAL256_H__

#include <limits.h>

#ifdef  __cplusplus
extern "C" {
#endif

    /*
     * We need an integer type with width 32-bit or more (preferably, with
     * a width of exactly 32 bits).
     */
#include <stdint.h>

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
        size_t ptr;
        uint32_t state[352];
        uint32_t Whigh, Wlow;
    } mshabal256_context;

  /*
   * Initialize a context structure. The output size is assumed to be
   * fixed to 256bit
   */
  void mshabal256_init(mshabal256_context *sc);

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
  void mshabal256(mshabal256_context *sc,
    const void *data0, const void *data1, const void *data2, const void *data3,
    const void *data4, const void *data5, const void *data6, const void *data7,
    size_t len);

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
   * size was specified as parameter to mshabal256_init()). It is acceptable
   * to use NULL for any of those pointers, if the result from the
   * corresponding instance is not needed.
   *
   * After this call, the context structure is invalid. The caller shall
   * release it, or reinitialize it with mshabal256_init(). The mshabal256_close()
   * function does NOT imply a hidden call to mshabal256_init().
   */
  void mshabal256_close(mshabal256_context *sc,
    uint32_t *dst0, uint32_t *dst1, uint32_t *dst2, uint32_t *dst3,
    uint32_t *dst4, uint32_t *dst5, uint32_t *dst6, uint32_t *dst7);

#ifdef  __cplusplus
}
#endif

#endif
