cmake_minimum_required(VERSION 3.15)
project(twsapi_builder LANGUAGES C CXX)

# Use relative paths from the project root
get_filename_component(PROJECT_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)

set(IBAPI_CLIENT_DIR "${PROJECT_ROOT}")
cmake_path(APPEND IBAPI_CLIENT_DIR "native" "third_party" "tws-api" "IBJts" "source" "cppclient" "client")

set(IBAPI_PROTO_DIR "${PROJECT_ROOT}")
cmake_path(APPEND IBAPI_PROTO_DIR "native" "third_party" "tws-api" "IBJts" "source" "cppclient" "client")

set(INTEL_BID_LIB "${PROJECT_ROOT}")
cmake_path(APPEND INTEL_BID_LIB "native" "third_party" "IntelRDFPMathLib20U2" "LIBRARY" "libbid.a")

# ------------------------------------------------------------------------------
# Dependency discovery
# ------------------------------------------------------------------------------
set(DEFAULT_PACKAGE_PREFIX "/usr/local")
if(APPLE AND EXISTS "/opt/homebrew")
  set(DEFAULT_PACKAGE_PREFIX "/opt/homebrew")
endif()

set(PACKAGE_PREFIX "${DEFAULT_PACKAGE_PREFIX}" CACHE PATH "Prefix where protobuf/abseil are installed")

find_path(PROTOBUF_INCLUDE_DIR
  NAMES google/protobuf/message_lite.h
  HINTS
    "${PACKAGE_PREFIX}/include"
    "/usr/local/include"
    "/opt/homebrew/include"
    "/usr/include"
)

find_library(PROTOBUF_LIBRARY
  NAMES protobuf libprotobuf
  HINTS
    "${PACKAGE_PREFIX}/lib"
    "/usr/local/lib"
    "/opt/homebrew/lib"
    "/usr/lib"
)

if(NOT PROTOBUF_INCLUDE_DIR OR NOT PROTOBUF_LIBRARY)
  message(FATAL_ERROR "Unable to locate protobuf headers or library. Set PROTOBUF_INCLUDE_DIR/PROTOBUF_LIBRARY.")
endif()

get_filename_component(PROTOBUF_LIB_DIR "${PROTOBUF_LIBRARY}" DIRECTORY)

find_library(PROTOBUF_LITE_LIBRARY
  NAMES protobuf-lite libprotobuf-lite
  HINTS
    "${PROTOBUF_LIB_DIR}"
    "${PACKAGE_PREFIX}/lib"
    "/usr/local/lib"
    "/opt/homebrew/lib"
    "/usr/lib"
)

# ------------------------------------------------------------------------------
# Source collection and targets
# ------------------------------------------------------------------------------
file(GLOB IBAPI_CORE_SOURCES "${IBAPI_CLIENT_DIR}/*.cpp")
file(GLOB IBAPI_PROTO_SOURCES "${IBAPI_PROTO_DIR}/*.pb.cc")

set(IBAPI_STDAFX "${IBAPI_CLIENT_DIR}/StdAfx.cpp")
list(REMOVE_ITEM IBAPI_CORE_SOURCES "${IBAPI_STDAFX}")

add_library(twsapi SHARED ${IBAPI_CORE_SOURCES} ${IBAPI_PROTO_SOURCES})

target_include_directories(twsapi
  PUBLIC
    "${IBAPI_CLIENT_DIR}"
    "${IBAPI_PROTO_DIR}"
    "${PROTOBUF_INCLUDE_DIR}"
)

target_compile_features(twsapi PUBLIC cxx_std_17)
target_compile_options(twsapi PRIVATE
    -Wno-unused-parameter
    -Wno-deprecated-declarations
)

target_link_libraries(twsapi
  PUBLIC
    "${PROTOBUF_LIBRARY}"
    "${INTEL_BID_LIB}"
    "-lz"
    "-lpthread"
)

if(PROTOBUF_LITE_LIBRARY)
  target_link_libraries(twsapi PUBLIC "${PROTOBUF_LITE_LIBRARY}")
endif()

# Link Abseil libraries if available (required by modern protobuf)
file(GLOB ABSEIL_LIBS
  "${PROTOBUF_LIB_DIR}/libabsl*.dylib"
  "${PROTOBUF_LIB_DIR}/libabsl*.a"
)
if(ABSEIL_LIBS)
  list(REMOVE_DUPLICATES ABSEIL_LIBS)
  target_link_libraries(twsapi PUBLIC ${ABSEIL_LIBS})
endif()

set_target_properties(twsapi PROPERTIES
  LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
  OUTPUT_NAME "TwsApiCpp"
  VERSION 1.0.0
  SOVERSION 1
)
