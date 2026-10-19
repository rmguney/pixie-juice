# Fuzzing

`fuzz_pixie.cpp` is a [libFuzzer](https://llvm.org/docs/LibFuzzer.html) harness for the whole engine API: the first byte of each input picks the call (optimize, convert, decode, decode_mesh, the HEIC decoder, convert_pixels) and its options, and the rest is the file. Every library is built with coverage instrumentation, AddressSanitizer and UndefinedBehaviorSanitizer, so memory errors and undefined behaviour anywhere in the engine stop the run.

`seeds.cpp` writes the starting corpus: small valid files of every format, each behind the mode bytes that exercise it.

CI runs it for 30 minutes whenever `core/` changes on `main`, or on demand (`.github/workflows/fuzz.yml`), keeping the corpus between runs. A crash fails the run and uploads the input that caused it. To reproduce one locally (Linux or macOS, clang 19 or later; with libstdc++, older clang has no `std::expected`):

```sh
cmake --preset fuzz && cmake --build --preset fuzz
../build/fuzz/pixie_fuzz crash-<hash>
```

To fuzz locally:

```sh
../build/fuzz/pixie_fuzz_seeds seeds
../build/fuzz/pixie_fuzz -max_len=262144 -timeout=60 corpus seeds
```

Inputs that only take long or need much memory (a large image is allowed up to 64 megapixels) are not bugs, so CI runs with `-ignore_timeouts=1 -ignore_ooms=1` in fork mode.
