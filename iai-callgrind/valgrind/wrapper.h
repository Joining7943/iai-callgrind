#include <stdbool.h>
#include <stddef.h>

#include "valgrind/callgrind.h"
#include "valgrind/valgrind.h"

#ifdef VALGRIND_DO_CLIENT_REQUEST_EXPR
const bool IC_IS_PLATFORM_SUPPORTED_BY_VALGRIND = true;
#else
const bool IC_IS_PLATFORM_SUPPORTED_BY_VALGRIND = false;
#endif

/*
 * VALGRIND
 * */

typedef enum {
#ifdef RUNNING_ON_VALGRIND
  IC_RUNNING_ON_VALGRIND = VG_USERREQ__RUNNING_ON_VALGRIND,
#else
  IC_RUNNING_ON_VALGRIND = 0,
#endif
#ifdef VALGRIND_DISCARD_TRANSLATIONS
  IC_VALGRIND_DISCARD_TRANSLATIONS = VG_USERREQ__DISCARD_TRANSLATIONS
#else
  IC_VALGRIND_DISCARD_TRANSLATIONS = 1
#endif
} IC_ValgrindClientRequest;

/*
 * CALLGRIND
 * */

typedef enum {
#ifdef CALLGRIND_DUMP_STATS
  IC_DUMP_STATS = VG_USERREQ__DUMP_STATS,
#else
  IC_DUMP_STATS = 0,
#endif
#ifdef CALLGRIND_DUMP_STATS_AT
  IC_DUMP_STATS_AT = VG_USERREQ__DUMP_STATS_AT,
#else
  IC_DUMP_STATS_AT = 1,
#endif
#ifdef CALLGRIND_ZERO_STATS
  IC_ZERO_STATS = VG_USERREQ__ZERO_STATS,
#else
  IC_ZERO_STATS = 2,
#endif
#ifdef CALLGRIND_TOGGLE_COLLECT
  IC_TOGGLE_COLLECT = VG_USERREQ__TOGGLE_COLLECT,
#else
  IC_TOGGLE_COLLECT = 3,
#endif
#ifdef CALLGRIND_START_INSTRUMENTATION
  IC_START_INSTRUMENTATION = VG_USERREQ__START_INSTRUMENTATION,
#else
  IC_START_INSTRUMENTATION = 4,
#endif
#ifdef CALLGRIND_STOP_INSTRUMENTATION
  IC_STOP_INSTRUMENTATION = VG_USERREQ__STOP_INSTRUMENTATION
#else
  IC_STOP_INSTRUMENTATION = 5
#endif
} IC_CallgrindClientRequest;
