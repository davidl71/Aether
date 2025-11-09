cmake_minimum_required(VERSION 3.15)
project(twsapi_builder LANGUAGES C CXX)

set(IBAPI_CLIENT_DIR "/Users/davidlowes/IBJts/source/cppclient")
cmake_path(APPEND IBAPI_CLIENT_DIR "client")

set(IB_DOWNLOADS_DIR "/Users/davidlowes/Downloads")
set(IB_BOX_SPREAD_ROOT "${IB_DOWNLOADS_DIR}")
cmake_path(APPEND IB_BOX_SPREAD_ROOT "ib_box_spread_full_universal")

set(IBAPI_PROTO_DIR "${IB_BOX_SPREAD_ROOT}")
cmake_path(APPEND IBAPI_PROTO_DIR "third_party" "ib" "protobuf")

set(PROTOBUF_SRC_DIR "${IB_BOX_SPREAD_ROOT}")
cmake_path(APPEND PROTOBUF_SRC_DIR "protobuf-3.20.3" "src")

set(PROTOBUF_BUILD_DIR "${IB_BOX_SPREAD_ROOT}")
cmake_path(APPEND PROTOBUF_BUILD_DIR "protobuf-build")

set(INTEL_BID_SRC_DIR "${IB_BOX_SPREAD_ROOT}")
cmake_path(APPEND INTEL_BID_SRC_DIR "third_party" "IntelRDFPMathLib20U2")
cmake_path(APPEND INTEL_BID_SRC_DIR "LIBRARY" "src")

file(GLOB IBAPI_CORE_SOURCES "${IBAPI_CLIENT_DIR}/*.cpp")
file(GLOB IBAPI_PROTO_SOURCES "${IBAPI_PROTO_DIR}/*.cc")
file(GLOB INTEL_BID_SOURCES "${INTEL_BID_SRC_DIR}/*.c")

set(IBAPI_STDAFX "${IBAPI_CLIENT_DIR}/StdAfx.cpp")
list(REMOVE_ITEM IBAPI_CORE_SOURCES "${IBAPI_STDAFX}")

add_library(bid_decimal STATIC ${INTEL_BID_SOURCES})
target_include_directories(bid_decimal PUBLIC "${INTEL_BID_SRC_DIR}")
target_compile_features(bid_decimal PUBLIC c_std_99)
target_compile_options(bid_decimal PRIVATE
    -std=gnu99
    -Wno-implicit-function-declaration
    -Wno-dangling-else
)
target_compile_definitions(bid_decimal PUBLIC
  DECIMAL_CALL_BY_REFERENCE=0
  DECIMAL_GLOBAL_ROUNDING=0
  DECIMAL_GLOBAL_EXCEPTION_FLAGS=0
  DECIMAL_GLOBAL_ROUNDING_FLAGS=0
  DECIMAL_GLOBAL_ROUNDING_EMULATION=0
)

add_library(twsapi SHARED ${IBAPI_CORE_SOURCES} ${IBAPI_PROTO_SOURCES})

target_include_directories(twsapi
  PUBLIC
    "${IBAPI_CLIENT_DIR}"
    "${IBAPI_PROTO_DIR}"
    "${PROTOBUF_SRC_DIR}"
)

target_compile_features(twsapi PUBLIC cxx_std_17)
target_compile_options(twsapi PRIVATE
    -Wno-unused-parameter
    -Wno-deprecated-declarations
)

target_link_libraries(twsapi
  PUBLIC
    "${PROTOBUF_BUILD_DIR}/libprotobuf.a"
    "${PROTOBUF_BUILD_DIR}/libprotobuf-lite.a"
    bid_decimal
    "-lz"
    "-lpthread"
)

set_target_properties(twsapi PROPERTIES
  LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
  OUTPUT_NAME "TwsApiCpp"
  VERSION 1.0.0
  SOVERSION 1
)
