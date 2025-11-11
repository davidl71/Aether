cmake_minimum_required(VERSION 3.15)
project(twsapi_builder LANGUAGES C CXX)

# Use relative paths from the project root
get_filename_component(PROJECT_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)

set(IBAPI_CLIENT_DIR "${PROJECT_ROOT}")
cmake_path(APPEND IBAPI_CLIENT_DIR "native" "third_party" "tws-api" "IBJts" "source" "cppclient" "client")

set(IBAPI_PROTO_DIR "${PROJECT_ROOT}")
cmake_path(APPEND IBAPI_PROTO_DIR "native" "third_party" "tws-api" "IBJts" "source" "cppclient" "client")

set(PROTOBUF_SRC_DIR "/usr/local/include")
set(PROTOBUF_BUILD_DIR "/usr/local/lib")

set(INTEL_BID_LIB "${PROJECT_ROOT}")
cmake_path(APPEND INTEL_BID_LIB "native" "third_party" "IntelRDFPMathLib20U2" "LIBRARY" "libbid.a")

file(GLOB IBAPI_CORE_SOURCES "${IBAPI_CLIENT_DIR}/*.cpp")
file(GLOB IBAPI_PROTO_SOURCES "${IBAPI_PROTO_DIR}/*.pb.cc")

set(IBAPI_STDAFX "${IBAPI_CLIENT_DIR}/StdAfx.cpp")
list(REMOVE_ITEM IBAPI_CORE_SOURCES "${IBAPI_STDAFX}")

add_library(twsapi SHARED ${IBAPI_CORE_SOURCES} ${IBAPI_PROTO_SOURCES})

target_include_directories(twsapi
  PUBLIC
    "${IBAPI_CLIENT_DIR}"
    "${IBAPI_PROTO_DIR}"
)

target_compile_features(twsapi PUBLIC cxx_std_17)
target_compile_options(twsapi PRIVATE
    -Wno-unused-parameter
    -Wno-deprecated-declarations
)

target_link_libraries(twsapi
  PUBLIC
    "${PROTOBUF_BUILD_DIR}/libprotobuf.dylib"
    "${INTEL_BID_LIB}"
    "-lz"
    "-lpthread"
)

# Link Abseil libraries if available (required by modern protobuf)
file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.dylib")
if(ABSEIL_LIBS)
  target_link_libraries(twsapi PUBLIC ${ABSEIL_LIBS})
endif()

set_target_properties(twsapi PROPERTIES
  LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
  OUTPUT_NAME "TwsApiCpp"
  VERSION 1.0.0
  SOVERSION 1
)
