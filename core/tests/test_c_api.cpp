#include "fixtures.hpp"

#include <doctest/doctest.h>

#include <pixie/pixie.h>

#include <string>
#include <thread>
#include <vector>

extern "C" int pixie_c_api_check(const std::uint8_t* png, std::size_t png_size, std::uint32_t width,
                                 std::uint32_t height);

TEST_CASE("C API: optimize, convert, decode and errors, called from C") {
  const auto image = fixtures::make_image(64, 48, 20);
  const auto png = pixie::detail::encode_png(image);
  REQUIRE(png.has_value());
  CHECK(pixie_c_api_check(png->data(), png->size(), 64, 48) == 0);
}

TEST_CASE("C API: calls run concurrently, including meshes staged in temporary files") {
  const auto ply = fixtures::text_bytes(
      "ply\nformat ascii 1.0\nelement vertex 4\nproperty float x\nproperty float y\nproperty float z\n"
      "element face 2\nproperty list uchar int vertex_indices\nend_header\n"
      "0 0 0\n1 0 0\n1 1 0\n0 1 0\n3 0 1 2\n3 0 2 3\n");
  const auto gltf = fixtures::grid_gltf(8);

  std::vector<std::string> failures(8);
  std::vector<std::thread> threads;
  for (std::size_t t = 0; t < failures.size(); ++t) {
    threads.emplace_back([&, t] {
      const auto& input = t % 2 ? ply : gltf;
      for (int round = 0; round < 4; ++round) {
        pixie_result result = pixie_optimize(input.data(), input.size(), nullptr);
        if (result.error) failures[t] = result.error;
        pixie_result_free(&result);
      }
    });
  }
  for (auto& thread : threads) thread.join();
  for (const auto& failure : failures) CHECK(failure.empty());
}
