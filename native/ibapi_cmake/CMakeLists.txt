cmake_minimum_required(VERSION 3.15)
project(twsapi_builder LANGUAGES C CXX)

# Use relative paths from the project root
get_filename_component(PROJECT_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)

set(IBAPI_CLIENT_DIR "${PROJECT_ROOT}")
cmake_path(APPEND IBAPI_CLIENT_DIR "native" "third_party" "tws-api" "IBJts" "source" "cppclient" "client")

set(IBAPI_PROTO_DIR "${PROJECT_ROOT}")
cmake_path(APPEND IBAPI_PROTO_DIR "native" "third_party" "tws-api" "IBJts" "source" "cppclient" "client")

# Platform-specific Protocol Buffers paths
if(WIN32)
    # Windows: typically installed via vcpkg or custom path
    set(PROTOBUF_SRC_DIR "${Protobuf_ROOT}/include" CACHE PATH "Protocol Buffers include directory")
    set(PROTOBUF_BUILD_DIR "${Protobuf_ROOT}/lib" CACHE PATH "Protocol Buffers library directory")
else()
    # macOS/Linux: typically in /usr/local
    set(PROTOBUF_SRC_DIR "/usr/local/include")
    set(PROTOBUF_BUILD_DIR "/usr/local/lib")
endif()

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

# Platform-specific linking
if(WIN32)
    # Windows: link against Winsock 2 and use .lib/.dll
    target_link_libraries(twsapi
      PUBLIC
        "${PROTOBUF_BUILD_DIR}/libprotobuf.lib"
        "${INTEL_BID_LIB}"
        "ws2_32"
    )
elseif(APPLE)
    # macOS: use .dylib and pthread
    target_link_libraries(twsapi
      PUBLIC
        "${PROTOBUF_BUILD_DIR}/libprotobuf.dylib"
        "${INTEL_BID_LIB}"
        "-lz"
        "-lpthread"
    )
else()
    # Linux: use .so and pthread
    target_link_libraries(twsapi
      PUBLIC
        "${PROTOBUF_BUILD_DIR}/libprotobuf.so"
        "${INTEL_BID_LIB}"
        "-lz"
        "-lpthread"
    )
endif()

# Link Abseil libraries if available (required by modern protobuf)
if(WIN32)
    # Windows: Abseil typically in vcpkg or custom path
    if(Protobuf_ROOT)
        file(GLOB ABSEIL_LIBS "${Protobuf_ROOT}/lib/absl*.lib")
    else()
        file(GLOB ABSEIL_LIBS "C:/vcpkg/installed/x64-windows/lib/absl*.lib")
    endif()
else()
    # macOS/Linux: typically in /usr/local/lib
    if(APPLE)
        file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.dylib")
    else()
        file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.so")
    endif()
endif()
if(ABSEIL_LIBS)
    target_link_libraries(twsapi PUBLIC ${ABSEIL_LIBS})
endif()

set_target_properties(twsapi PROPERTIES
  LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
  OUTPUT_NAME "TwsApiCpp"
  VERSION 1.0.0
  SOVERSION 1
)
