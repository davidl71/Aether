// test_path_validator.cpp - Tests for path validation security utility
#include <catch2/catch_test_macros.hpp>
#include "path_validator.h"
#include <filesystem>
#include <vector>

using namespace box_spread::security;

TEST_CASE("PathValidator - Basic validation", "[security][path_validator]") {
  std::vector<std::filesystem::path> allowed_paths = {
    std::filesystem::current_path(),
    std::filesystem::current_path() / "allowed_dir"
  };
  
  PathValidator validator(allowed_paths);
  
  SECTION("Valid path within allowed directory") {
    auto result = validator.validate_path("allowed_dir/test.txt");
    REQUIRE(result.has_value());
    REQUIRE(result->filename() == "test.txt");
  }
  
  SECTION("Invalid path with directory traversal") {
    auto result = validator.validate_path("../outside.txt");
    REQUIRE_FALSE(result.has_value());
  }
  
  SECTION("Invalid path with encoded traversal") {
    auto result = validator.validate_path("test%2e%2e%2foutside.txt");
    REQUIRE_FALSE(result.has_value());
  }
}

TEST_CASE("PathValidator - Traversal detection", "[security][path_validator]") {
  SECTION("Detects ../ pattern") {
    REQUIRE(PathValidator::contains_traversal(std::filesystem::path("../test.txt")));
  }
  
  SECTION("Detects ..\\ pattern") {
    REQUIRE(PathValidator::contains_traversal(std::filesystem::path("..\\test.txt")));
  }
  
  SECTION("Detects encoded traversal") {
    REQUIRE(PathValidator::contains_traversal(std::filesystem::path("test%2e%2e%2foutside.txt")));
  }
  
  SECTION("Does not detect false positives") {
    REQUIRE_FALSE(PathValidator::contains_traversal(std::filesystem::path("test.txt")));
    REQUIRE_FALSE(PathValidator::contains_traversal(std::filesystem::path("dir/test.txt")));
  }
}

TEST_CASE("PathValidator - Default validator", "[security][path_validator]") {
  auto validator = create_default_path_validator();
  
  SECTION("Allows current directory") {
    auto result = validator.validate_path("test.txt");
    // Should allow if within current directory
    // Result depends on current working directory
    // Just verify it doesn't crash
    (void)result;
  }
  
  SECTION("Allows home directory") {
    const char* home = std::getenv("HOME");
    if (home) {
      auto result = validator.validate_path(std::filesystem::path(home) / ".config" / "ib_box_spread" / "test.json");
      // Should allow if within allowed paths
      // Just verify it doesn't crash
      (void)result;
    }
  }
}
