#pragma once

// Checks on optimize results, for the test suites (fixtures.hpp is shared with
// the fuzzer's seeds, which don't use doctest).

#include "fixtures.hpp"

#include <doctest/doctest.h>

namespace fixtures {

/// The result file: the new data, or the input itself when nothing changed.
inline pixie::Output optimized(const Bytes& input, pixie::Options options = {}) {
  auto result = pixie::optimize(input, options);
  REQUIRE(result.has_value());
  if (!result->changed) {
    CHECK(result->data.empty());
    result->data = input;
  }
  return *result;
}

}  // namespace fixtures
