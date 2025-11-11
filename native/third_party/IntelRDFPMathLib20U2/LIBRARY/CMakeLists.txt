cmake_minimum_required(VERSION 3.15)
project(bid_decimal LANGUAGES C)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Detect architecture for float128
if(APPLE)
  if(CMAKE_SYSTEM_PROCESSOR MATCHES "arm64|aarch64")
    set(ARCHITECTURE "amd64")
    set(BITS_PER_WORD 64)
  else()
    set(ARCHITECTURE "amd64")
    set(BITS_PER_WORD 64)
  endif()
elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "x86_64|AMD64")
  set(ARCHITECTURE "amd64")
  set(BITS_PER_WORD 64)
elseif(CMAKE_SYSTEM_PROCESSOR MATCHES "i386|i686|x86")
  set(ARCHITECTURE "ix86")
  set(BITS_PER_WORD 32)
endif()

# Set compile definitions based on TWS API requirements
add_compile_definitions(
  DECIMAL_CALL_BY_REFERENCE=0
  DECIMAL_GLOBAL_ROUNDING=0
  DECIMAL_GLOBAL_EXCEPTION_FLAGS=0
  DECIMAL_GLOBAL_ROUNDING_FLAGS=0
  DECIMAL_GLOBAL_ROUNDING_EMULATION=0
  ARCHITECTURE=${ARCHITECTURE}
  BITS_PER_WORD=${BITS_PER_WORD}
  BITS_PER_CHAR=8
  BITS_PER_SHORT=16
  BITS_PER_INT=32
  BITS_PER_LONG=64
  BITS_PER_FLOAT=32
  BITS_PER_DOUBLE=64
  BITS_PER_LONG_DOUBLE=128
)

# Get all source files - exclude float128 for now as it requires complex platform setup
file(GLOB BID_SOURCES "src/*.c")
# file(GLOB FLOAT128_SOURCES "float128/*.c")  # Excluded for now

# Create static library
add_library(bid_decimal STATIC ${BID_SOURCES})

target_include_directories(bid_decimal PUBLIC
  "${CMAKE_CURRENT_SOURCE_DIR}/src"
)

target_compile_options(bid_decimal PRIVATE
  -std=gnu99
  -Wno-implicit-function-declaration
  -Wno-dangling-else
)

# Set output name to match expected libbid.a
set_target_properties(bid_decimal PROPERTIES
  OUTPUT_NAME "bid"
  ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
)

