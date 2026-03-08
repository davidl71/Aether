cmake_minimum_required(VERSION 3.15)
project(twsapi_builder LANGUAGES C CXX)

# ============================================================================
# macOS SDK / Toolchain Fix (mirror of root native/CMakeLists.txt)
# When CLT C++ headers are missing, inject SDK C++ path so TWS API build finds
# <string>, <mutex>, etc. Required when ibapi_cmake runs as ExternalProject or standalone.
# ============================================================================
if(APPLE)
  if(NOT CMAKE_OSX_SYSROOT)
    execute_process(
      COMMAND xcrun --show-sdk-path
      OUTPUT_VARIABLE _DETECTED_SDK
      OUTPUT_STRIP_TRAILING_WHITESPACE
      ERROR_QUIET
    )
    if(_DETECTED_SDK)
      set(CMAKE_OSX_SYSROOT "${_DETECTED_SDK}" CACHE PATH "macOS SDK path")
    endif()
  endif()
  if(CMAKE_OSX_SYSROOT)
    set(_SDK_CXX "${CMAKE_OSX_SYSROOT}/usr/include/c++/v1")
    set(_TC_CXX "/Library/Developer/CommandLineTools/usr/include/c++/v1")
    if(EXISTS "${_SDK_CXX}/string" AND NOT EXISTS "${_TC_CXX}/string")
      set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -stdlib++-isystem${_SDK_CXX}" CACHE STRING "" FORCE)
      message(STATUS "TWS API: Injecting SDK C++ headers (CLT headers missing): ${_SDK_CXX}")
    elseif(EXISTS "${_SDK_CXX}/string")
      set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -stdlib++-isystem${_SDK_CXX}" CACHE STRING "" FORCE)
      message(STATUS "TWS API: Injecting SDK C++ headers for subprojects: ${_SDK_CXX}")
    elseif(NOT EXISTS "${_TC_CXX}/string")
      message(WARNING "TWS API: C++ stdlib headers not found in CLT or SDK. Run: xcode-select --install")
    endif()
  endif()
endif()

# Use relative paths from the project root when TWS_API_CLIENT_DIR not provided (zip layout).
get_filename_component(PROJECT_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)

if(DEFINED TWS_API_CLIENT_DIR AND TWS_API_CLIENT_DIR)
  set(IBAPI_CLIENT_DIR "${TWS_API_CLIENT_DIR}")
  # GitHub layout: proto sources in source/proto; generate .pb.cc into build dir
  get_filename_component(IBAPI_PROTO_SOURCE "${IBAPI_CLIENT_DIR}/../../proto" REALPATH)
  set(IBAPI_PROTO_DIR "${IBAPI_PROTO_SOURCE}")
  message(STATUS "TWS API client directory (GitHub layout): ${IBAPI_CLIENT_DIR}")
  message(STATUS "TWS API proto source: ${IBAPI_PROTO_SOURCE}")
else()
  set(IBAPI_CLIENT_DIR "${PROJECT_ROOT}")
  cmake_path(APPEND IBAPI_CLIENT_DIR "native" "third_party" "tws-api" "IBJts" "source" "cppclient" "client")
  set(IBAPI_PROTO_DIR "${PROJECT_ROOT}")
  cmake_path(APPEND IBAPI_PROTO_DIR "native" "third_party" "tws-api" "IBJts" "source" "cppclient" "client")
endif()

# When we will generate .pb.h (GitHub layout), stage client sources without .pb.h so that
# #include "*.pb.h" resolves to our generated headers (compiler searches current file dir first).
set(IBAPI_CLIENT_SOURCES_DIR "${IBAPI_CLIENT_DIR}")
if(DEFINED TWS_API_CLIENT_DIR AND TWS_API_CLIENT_DIR AND EXISTS "${IBAPI_PROTO_SOURCE}")
  set(_client_staging "${CMAKE_CURRENT_BINARY_DIR}/client_staging")
  file(MAKE_DIRECTORY "${_client_staging}")
  file(GLOB _client_cpp "${IBAPI_CLIENT_DIR}/*.cpp")
  file(GLOB _client_h "${IBAPI_CLIENT_DIR}/*.h")
  foreach(_f ${_client_cpp} ${_client_h})
    get_filename_component(_name "${_f}" NAME)
    if(NOT _name MATCHES "\\.pb\\.h$")
      configure_file("${_f}" "${_client_staging}/${_name}" COPYONLY)
    endif()
  endforeach()
  set(IBAPI_CLIENT_SOURCES_DIR "${_client_staging}")
  message(STATUS "TWS API: client staging (no .pb.h) ${_client_staging}")
  unset(_client_staging)
  unset(_client_cpp)
  unset(_client_h)
  unset(_f)
  unset(_name)
endif()

# Platform-specific Protocol Buffers paths
if(WIN32)
    # Windows: typically installed via vcpkg or custom path
    set(PROTOBUF_SRC_DIR "${Protobuf_ROOT}/include" CACHE PATH "Protocol Buffers include directory")
    set(PROTOBUF_BUILD_DIR "${Protobuf_ROOT}/lib" CACHE PATH "Protocol Buffers library directory")
else()
    # macOS/Linux: allow Protobuf_ROOT override (e.g. -DProtobuf_ROOT=/opt/homebrew), else discover
    if(Protobuf_ROOT)
        set(PROTOBUF_SRC_DIR "${Protobuf_ROOT}/include" CACHE PATH "Protocol Buffers include directory")
        set(PROTOBUF_BUILD_DIR "${Protobuf_ROOT}/lib" CACHE PATH "Protocol Buffers library directory")
    else()
        set(PROTOBUF_SRC_DIR "/usr/local/include")
        set(PROTOBUF_BUILD_DIR "/usr/local/lib")
    endif()
    # Prefer find_library so versioned names (e.g. libprotobuf.32.dylib) and Homebrew are found
    find_library(Protobuf_LIBRARY
        NAMES protobuf libprotobuf
        PATHS "${PROTOBUF_BUILD_DIR}" /opt/homebrew/lib /usr/local/lib /usr/lib
        NO_DEFAULT_PATH
    )
    if(NOT Protobuf_LIBRARY)
        find_library(Protobuf_LIBRARY NAMES protobuf libprotobuf)
    endif()
    if(Protobuf_LIBRARY)
        get_filename_component(PROTOBUF_BUILD_DIR "${Protobuf_LIBRARY}" DIRECTORY)
        message(STATUS "Found protobuf: ${Protobuf_LIBRARY}")
        # Keep PROTOBUF_SRC_DIR in sync with library prefix when Protobuf_ROOT was not set
        if(NOT Protobuf_ROOT)
            get_filename_component(_pb_prefix "${Protobuf_LIBRARY}" DIRECTORY)
            get_filename_component(_pb_prefix "${_pb_prefix}" DIRECTORY)
            set(PROTOBUF_SRC_DIR "${_pb_prefix}/include")
            if(EXISTS "${PROTOBUF_SRC_DIR}/google/protobuf")
                message(STATUS "TWS API: Protobuf include from library prefix: ${PROTOBUF_SRC_DIR}")
            endif()
            unset(_pb_prefix)
        endif()
    endif()
endif()

set(INTEL_BID_LIB "${PROJECT_ROOT}")
set(_ibapi_u4 "${PROJECT_ROOT}/native/third_party/IntelRDFPMathLib20U4/LIBRARY/libbid.a")
set(_ibapi_u2 "${PROJECT_ROOT}/native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a")
if(EXISTS "${_ibapi_u4}")
  set(INTEL_BID_LIB "${_ibapi_u4}")
elseif(EXISTS "${_ibapi_u2}")
  set(INTEL_BID_LIB "${_ibapi_u2}")
else()
  cmake_path(APPEND INTEL_BID_LIB "native" "third_party" "IntelRDFPMathLib20U4" "LIBRARY" "libbid.a")
endif()
unset(_ibapi_u4)
unset(_ibapi_u2)

file(GLOB IBAPI_CORE_SOURCES "${IBAPI_CLIENT_SOURCES_DIR}/*.cpp")

# GitHub layout: generate .pb.cc from source/proto; zip layout: use existing .pb.cc in client
set(IBAPI_PROTO_SOURCES "")
set(IBAPI_PROTO_GENERATED_DIR "")
if(DEFINED TWS_API_CLIENT_DIR AND TWS_API_CLIENT_DIR AND EXISTS "${IBAPI_PROTO_SOURCE}")
  find_package(Protobuf REQUIRED)
  # TWS API source/ProtoBuf_readme.txt: Linux 3.12.4, Windows 5.29.5
  if(Protobuf_VERSION)
    if(UNIX AND NOT APPLE)
      if(Protobuf_VERSION VERSION_LESS "3.12")
        message(WARNING "TWS API recommends Protobuf 3.12+ on Linux. Found: ${Protobuf_VERSION}. See source/ProtoBuf_readme.txt")
      endif()
    elseif(WIN32)
      if(Protobuf_VERSION VERSION_LESS "5")
        message(WARNING "TWS API recommends Protobuf 5.x on Windows. Found: ${Protobuf_VERSION}. See source/ProtoBuf_readme.txt")
      endif()
    endif()
  endif()
  file(GLOB IBAPI_PROTO_FILES "${IBAPI_PROTO_SOURCE}/*.proto")
  if(IBAPI_PROTO_FILES)
    set(PROTOBUF_OUTPUT_DIR "${CMAKE_CURRENT_BINARY_DIR}/proto_generated")
    file(MAKE_DIRECTORY "${PROTOBUF_OUTPUT_DIR}")
    foreach(_proto ${IBAPI_PROTO_FILES})
      get_filename_component(_name ${_proto} NAME_WE)
      set(_pb_cpp "${PROTOBUF_OUTPUT_DIR}/${_name}.pb.cc")
      set(_pb_h "${PROTOBUF_OUTPUT_DIR}/${_name}.pb.h")
      add_custom_command(
        OUTPUT "${_pb_cpp}" "${_pb_h}"
        COMMAND ${Protobuf_PROTOC_EXECUTABLE}
        ARGS --proto_path="${IBAPI_PROTO_SOURCE}"
             --cpp_out="${PROTOBUF_OUTPUT_DIR}"
             --experimental_allow_proto3_optional
             "${_proto}"
        DEPENDS "${_proto}"
        COMMENT "Generating ${_name}.pb.cc"
      )
      list(APPEND IBAPI_PROTO_SOURCES "${_pb_cpp}")
    endforeach()
    set(IBAPI_PROTO_GENERATED_DIR "${PROTOBUF_OUTPUT_DIR}")
  endif()
else()
  file(GLOB IBAPI_PROTO_SOURCES "${IBAPI_PROTO_DIR}/*.pb.cc")
endif()

set(IBAPI_STDAFX "${IBAPI_CLIENT_SOURCES_DIR}/StdAfx.cpp")
list(REMOVE_ITEM IBAPI_CORE_SOURCES "${IBAPI_STDAFX}")

add_library(twsapi SHARED ${IBAPI_CORE_SOURCES} ${IBAPI_PROTO_SOURCES})

# When we regenerate .pb.h from .proto (GitHub/vendored layout), put generated dir first
# so the build uses headers matching the system protobuf instead of vendored 6.33.4 headers.
# Generated .pb.h include google/protobuf/runtime_version.h etc., so add Protobuf include dir.
# We use IBAPI_CLIENT_SOURCES_DIR (staging without .pb.h when generating) so #include "*.pb.h" finds generated ones.
if(IBAPI_PROTO_GENERATED_DIR)
  target_include_directories(twsapi
    PUBLIC
      "${IBAPI_PROTO_GENERATED_DIR}"
      "${IBAPI_CLIENT_SOURCES_DIR}"
      "${IBAPI_PROTO_DIR}"
  )
else()
  target_include_directories(twsapi
    PUBLIC
      "${IBAPI_CLIENT_SOURCES_DIR}"
      "${IBAPI_PROTO_DIR}"
  )
endif()
if(PROTOBUF_SRC_DIR AND EXISTS "${PROTOBUF_SRC_DIR}/google/protobuf")
  # Verify runtime header exists (required by generated .pb.h)
  if(NOT EXISTS "${PROTOBUF_SRC_DIR}/google/protobuf/runtime_version.h")
    message(WARNING "TWS API: ${PROTOBUF_SRC_DIR}/google/protobuf/runtime_version.h not found; .pb.h may fail to compile")
  else()
    message(STATUS "TWS API: Protobuf include ${PROTOBUF_SRC_DIR} (runtime_version.h verified)")
  endif()
  target_include_directories(twsapi PUBLIC "${PROTOBUF_SRC_DIR}")
elseif(Protobuf_INCLUDE_DIRS)
  # Fallback when PROTOBUF_SRC_DIR not set (e.g. ExternalProject with different discovery)
  target_include_directories(twsapi PUBLIC ${Protobuf_INCLUDE_DIRS})
  message(STATUS "TWS API: Protobuf include from find_package: ${Protobuf_INCLUDE_DIRS}")
elseif(IBAPI_PROTO_GENERATED_DIR)
  message(WARNING "TWS API: PROTOBUF_SRC_DIR and Protobuf_INCLUDE_DIRS missing; generated .pb.h may fail to compile")
endif()

# macOS: add SDK C++ stdlib include so TWS API finds <string>, <mutex>, etc. (CLT headers)
if(APPLE)
  set(_sdk_root "${CMAKE_OSX_SYSROOT}")
  if(NOT _sdk_root)
    execute_process(
      COMMAND xcrun --show-sdk-path
      OUTPUT_VARIABLE _sdk_root
      OUTPUT_STRIP_TRAILING_WHITESPACE
      ERROR_QUIET
    )
  endif()
  if(_sdk_root)
    set(_sdk_cxx "${_sdk_root}/usr/include/c++/v1")
    if(EXISTS "${_sdk_cxx}/string")
      target_include_directories(twsapi PRIVATE "${_sdk_cxx}")
      message(STATUS "TWS API: using SDK C++ include ${_sdk_cxx}")
    endif()
  endif()
  unset(_sdk_root)
  unset(_sdk_cxx)
endif()

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
    # macOS: use found library or fallback path (find_library gets versioned dylib e.g. libprotobuf.32.dylib)
    if(Protobuf_LIBRARY)
        target_link_libraries(twsapi PUBLIC "${Protobuf_LIBRARY}")
    else()
        target_link_libraries(twsapi PUBLIC "${PROTOBUF_BUILD_DIR}/libprotobuf.dylib")
    endif()
    target_link_libraries(twsapi PUBLIC "${INTEL_BID_LIB}" "-lz" "-lpthread")
else()
    # Linux: use found library or fallback path
    if(Protobuf_LIBRARY)
        target_link_libraries(twsapi PUBLIC "${Protobuf_LIBRARY}")
    else()
        target_link_libraries(twsapi PUBLIC "${PROTOBUF_BUILD_DIR}/libprotobuf.so")
    endif()
    target_link_libraries(twsapi PUBLIC "${INTEL_BID_LIB}" "-lz" "-lpthread")
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
    # macOS/Linux: search same prefix as protobuf, then common paths
    set(_abseil_search_dirs "${PROTOBUF_BUILD_DIR}" /opt/homebrew/lib /usr/local/lib /usr/lib)
    foreach(_dir ${_abseil_search_dirs})
        if(APPLE)
            file(GLOB _absl "${_dir}/libabsl*.dylib")
        else()
            file(GLOB _absl "${_dir}/libabsl*.so")
        endif()
        if(_absl)
            set(ABSEIL_LIBS ${_absl})
            break()
        endif()
    endforeach()
endif()
if(ABSEIL_LIBS)
    target_link_libraries(twsapi PUBLIC ${ABSEIL_LIBS})
endif()

set_target_properties(twsapi PROPERTIES
  LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
  OUTPUT_NAME "twsapi"
  DEBUG_POSTFIX "d"
  VERSION 1.0.0
  SOVERSION 1
)
