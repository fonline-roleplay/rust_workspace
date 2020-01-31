#define SKIP_PRAGMAS
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wmacro-redefined"
#pragma GCC diagnostic ignored "-Wduplicate-decl-specifier"
#pragma GCC diagnostic ignored "-Wignored-attributes"
#pragma GCC diagnostic ignored "-Wlogical-op-parentheses"
#pragma GCC diagnostic ignored "-Wbitwise-op-parentheses"
#pragma GCC diagnostic ignored "-Wimplicit-exception-spec-mismatch"
#pragma GCC diagnostic ignored "-Wdelete-incomplete"
#pragma GCC diagnostic ignored "-Wnonportable-include-path"
#include "_defines.fos"
#include "fonline.h"
/*
template<class T>
struct stlp_std_vector {
    vector<T> inner;
};*/
void bindgen_static_asserts() {
    STATIC_ASSERT(sizeof(UintPair) == 8);
    STATIC_ASSERT(sizeof(Uint16Pair) == 4);
    STATIC_ASSERT(sizeof(string) == 28);
    STATIC_ASSERT(sizeof(ScriptString) == 36);
    STATIC_ASSERT(sizeof(ScriptArray) == 28);
    STATIC_ASSERT(sizeof(IntVec) == 16);
    STATIC_ASSERT(sizeof(CrVec) == 16);
    STATIC_ASSERT(sizeof(IntSet) == 16);
    STATIC_ASSERT(sizeof(CrMap) == 16);
    STATIC_ASSERT(sizeof(Critter) == 9336);
    STATIC_ASSERT(sizeof(GameOptions) == 1072);

#ifdef __SERVER
    STATIC_ASSERT( sizeof( Item ) == 196 );
#endif
#ifdef __CLIENT
    STATIC_ASSERT( sizeof( Item ) == 152 );
#endif
}
#pragma GCC diagnostic pop
