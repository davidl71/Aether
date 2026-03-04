// test_main.cpp - Main test file (Catch2 will provide main)
// This file can be empty when using Catch2::Catch2WithMain
// or you can customize test configuration here

#include <catch2/catch_test_macros.hpp>

// Optional: Global test setup/teardown
struct GlobalTestSetup {
  GlobalTestSetup() {
    // Setup code that runs once before all tests
  }

  ~GlobalTestSetup() {
    // Cleanup code that runs once after all tests
  }
};

static GlobalTestSetup global_setup;
