# Patch script for FetchContent PATCH_COMMAND.
# Invoked with working directory = fetched project source dir.
# Replaces cmake_minimum_required(VERSION x.y or x.y.z) with VERSION 3.10 (i.e. >= 3.10)
# to silence: "Compatibility with CMake < 3.10 will be removed from a future version of CMake."
if(NOT EXISTS "CMakeLists.txt")
  message(FATAL_ERROR "patch_cmake_minimum: CMakeLists.txt not found in working directory")
endif()
file(READ "CMakeLists.txt" _content)
string(REGEX REPLACE
  "cmake_minimum_required\\(VERSION [0-9]+\\.[0-9]+(\\.[0-9]+)?\\)"
  "cmake_minimum_required(VERSION 3.10)"
  _content "${_content}")
file(WRITE "CMakeLists.txt" "${_content}")
