# Expand user-supplied tilde paths to absolute paths for IB API overrides.
function(_expand_user_path INPUT_VAR OUTPUT_VAR)
    set(_value "${${INPUT_VAR}}")
    if(NOT _value)
        set(${OUTPUT_VAR} "" PARENT_SCOPE)
        return()
    endif()

    file(TO_CMAKE_PATH "${_value}" _normalized_value)
    if(_normalized_value MATCHES "^~")
        if(DEFINED ENV{HOME} AND NOT "$ENV{HOME}" STREQUAL "")
            set(_home "$ENV{HOME}")
            string(
                REGEX REPLACE "^~" "${_home}"
                _normalized_value "${_normalized_value}"
            )
        endif()
    endif()

    if(IS_ABSOLUTE "${_normalized_value}")
        set(_resolved "${_normalized_value}")
    else()
        get_filename_component(
            _resolved "${_normalized_value}" REALPATH
            BASE_DIR "${CMAKE_CURRENT_SOURCE_DIR}"
        )
    endif()

    set(${OUTPUT_VAR} "${_resolved}" PARENT_SCOPE)
endfunction()

cmake_minimum_required(VERSION 3.21)

# Suppress CMP0167 warning from QuantLib's find_package(Boost) (CMake 3.30+ removed FindBoost;
# we keep OLD behavior so FetchContent QuantLib builds without warning).
if(POLICY CMP0167)
  cmake_policy(SET CMP0167 OLD)
endif()
# Suppress any remaining developer/policy warnings from FetchContent subprojects (e.g. QuantLib).
if(CMAKE_VERSION VERSION_GREATER_EQUAL "3.25")
  set(CMAKE_SUPPRESS_DEVELOPER_WARNINGS ON)
endif()

# ============================================================================
# macOS SDK / Toolchain Fix (must be before project() for correct compiler checks)
# On macOS 26 (Tahoe) and some CLT installs, the toolchain may ship without
# C++ stdlib headers in /Library/Developer/CommandLineTools/usr/include/c++/v1.
# Detect this and inject the SDK header path so that try_compile and builds succeed.
# If builds still fail with missing <string> etc., run: xcode-select --install
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
    # Fix incomplete C++ stdlib in toolchain: use SDK C++ headers when CLT path lacks them
    if(CMAKE_OSX_SYSROOT)
        set(_SDK_CXX "${CMAKE_OSX_SYSROOT}/usr/include/c++/v1")
        set(_TC_CXX "/Library/Developer/CommandLineTools/usr/include/c++/v1")
        if(EXISTS "${_SDK_CXX}/string" AND NOT EXISTS "${_TC_CXX}/string")
            set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -stdlib++-isystem${_SDK_CXX}" CACHE STRING "" FORCE)
            message(STATUS "Injecting SDK C++ headers (CLT headers missing): ${_SDK_CXX}")
        elseif(EXISTS "${_SDK_CXX}/string")
            # SDK has C++ headers; add for subprojects (e.g. FetchContent NLopt) that run try_compile without CLT path
            set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -stdlib++-isystem${_SDK_CXX}" CACHE STRING "" FORCE)
            message(STATUS "Injecting SDK C++ headers for subprojects: ${_SDK_CXX}")
        elseif(NOT EXISTS "${_TC_CXX}/string")
            message(WARNING "C++ stdlib headers not found in CLT or SDK. Run: xcode-select --install")
        endif()
    endif()
endif()

# ============================================================================
# Project Configuration
# ============================================================================
project(SyntheticFinancingPlatform
    VERSION 1.3.3
    DESCRIPTION "Comprehensive multi-asset financing optimization system. Box spreads are one strategy component of this platform."
    LANGUAGES CXX
)

# ============================================================================
# Build Options
# ============================================================================
option(BUILD_TESTING "Build test suite" ON)
option(BUILD_DOCS "Build documentation" OFF)
option(ENABLE_ASAN "Enable AddressSanitizer" OFF)
option(ENABLE_TSAN "Enable ThreadSanitizer" OFF)
option(ENABLE_LTO "Enable Link-Time Optimization" ON)
option(ENABLE_PYTHON_BINDINGS "Enable Python bindings (Cython or pybind11)" ON)
option(PYTHON_BINDINGS_BACKEND "Python bindings backend: pybind11 or Cython" "pybind11")
option(ENABLE_DISTCC "Enable distcc distributed compilation" OFF)
option(ENABLE_CCACHE "Enable ccache compilation caching" OFF)
option(ENABLE_SCCACHE "Enable sccache compilation caching" OFF)
option(ENABLE_NATIVE_CLI "Build the native ib_box_spread CLI" ON)
option(TWS_API_BUILD_VENDOR "Build TWS API library from source (ibapi_cmake)" OFF)
option(BUILD_AI_FRIENDLY "Quieter build + JSON compiler diagnostics for tools/AI" OFF)

# ============================================================================
# Early dependency check (fail fast before FetchContent / heavy configure)
# When ENABLE_NATIVE_CLI is ON, require TWS API and Intel decimal to exist so
# configure fails quickly with a clear hint instead of failing later.
# ============================================================================
if(ENABLE_NATIVE_CLI)
  set(_tws_found OFF)
  if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/third_party/tws-api/source/cppclient/client"
      OR EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/third_party/tws-api/IBJts")
    set(_tws_found ON)
  endif()
  if(NOT _tws_found AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/../tws-api/source/cppclient/client")
    set(_tws_found ON)
  endif()
  if(NOT _tws_found AND TWS_API_SOURCE_DIR
      AND EXISTS "${TWS_API_SOURCE_DIR}/source/cppclient/client")
    set(_tws_found ON)
  endif()

  set(_intel_found OFF)
  if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a"
      OR EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/third_party/IntelRDFPMathLib20U4/LIBRARY/libbid.a")
    set(_intel_found ON)
  endif()

  if(NOT _tws_found OR NOT _intel_found)
    set(_early_missing "")
    if(NOT _tws_found)
      list(APPEND _early_missing "TWS API")
    endif()
    if(NOT _intel_found)
      list(APPEND _early_missing "Intel decimal (libbid.a)")
    endif()
    string(REPLACE ";" ", " _early_missing_str "${_early_missing}")
    message(FATAL_ERROR
      "Native CLI requires: ${_early_missing_str}.\n"
      "Run from repo root: ./scripts/fetch_third_party.sh\n"
      "Then reconfigure. See README for TWS API and Intel decimal setup.")
  endif()
endif()

# ============================================================================
# C++ Standard and Compiler Requirements
# ============================================================================
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Export compile commands for IDE support
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Silence CMake deprecation/developer warnings (policies)
if(POLICY CMP0135)
  cmake_policy(SET CMP0135 NEW)  # FetchContent: use extraction time for URL downloads
endif()
if(POLICY CMP0167)
  cmake_policy(SET CMP0167 OLD)  # Keep using FindBoost until BoostConfig is used everywhere
endif()
if(POLICY CMP0146)
  cmake_policy(SET CMP0146 OLD)  # Keep FindCUDA in Eigen tests; suppresses deprecation in deps
endif()

# ============================================================================
# Distributed Compilation and Caching
# ============================================================================
# Prioritize: sccache > ccache > distcc
if(ENABLE_SCCACHE)
    find_program(SCCACHE_EXECUTABLE sccache)
    if(SCCACHE_EXECUTABLE)
        message(STATUS "Using sccache: ${SCCACHE_EXECUTABLE}")
        set(CMAKE_CXX_COMPILER_LAUNCHER ${SCCACHE_EXECUTABLE})
        set(CMAKE_C_COMPILER_LAUNCHER ${SCCACHE_EXECUTABLE})
    else()
        message(WARNING "sccache not found, falling back to ccache/distcc")
        set(ENABLE_SCCACHE OFF)
    endif()
endif()

if(ENABLE_CCACHE AND NOT ENABLE_SCCACHE)
    find_program(CCACHE_EXECUTABLE ccache)
    if(CCACHE_EXECUTABLE)
        message(STATUS "Using ccache: ${CCACHE_EXECUTABLE}")
        set(CMAKE_CXX_COMPILER_LAUNCHER ${CCACHE_EXECUTABLE})
        set(CMAKE_C_COMPILER_LAUNCHER ${CCACHE_EXECUTABLE})

        if(ENABLE_DISTCC)
            find_program(DISTCC_EXECUTABLE distcc)
            if(DISTCC_EXECUTABLE)
                message(STATUS "Using ccache with distcc backend")
                set(ENV{CCACHE_PREFIX} "distcc")
            endif()
        endif()
    else()
        message(WARNING "ccache not found, trying distcc only")
        set(ENABLE_CCACHE OFF)
    endif()
endif()

if(ENABLE_DISTCC AND NOT ENABLE_CCACHE AND NOT ENABLE_SCCACHE)
    find_program(DISTCC_EXECUTABLE distcc)
    if(DISTCC_EXECUTABLE)
        message(STATUS "Using distcc: ${DISTCC_EXECUTABLE}")
        set(CMAKE_CXX_COMPILER_LAUNCHER ${DISTCC_EXECUTABLE})
        set(CMAKE_C_COMPILER_LAUNCHER ${DISTCC_EXECUTABLE})
    else()
        message(WARNING "distcc not found, distributed compilation disabled")
    endif()
endif()

# ============================================================================
# Output Directories
# ============================================================================
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# ============================================================================
# Compiler Flags
# ============================================================================
if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
    add_compile_options(
        -Wall
        -Wextra
        -Wpedantic
        -Wconversion
        -Wsign-conversion
        -Wshadow
        -Wnon-virtual-dtor
        -Wold-style-cast
        -Wcast-align
        -Wunused
        -Woverloaded-virtual
        -Wnull-dereference
        -Wdouble-promotion
        -Wformat=2
    )

    if(CMAKE_BUILD_TYPE MATCHES "Release")
        add_compile_options(-O3)

        if(ENABLE_LTO)
            set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
        endif()
    endif()

    if(ENABLE_ASAN)
        add_compile_options(-fsanitize=address -fno-omit-frame-pointer)
        add_link_options(-fsanitize=address)
    endif()

    if(ENABLE_TSAN)
        add_compile_options(-fsanitize=thread)
        add_link_options(-fsanitize=thread)
    endif()

    # AI-friendly: emit compiler diagnostics as JSON (parseable by tools/AI)
    if(BUILD_AI_FRIENDLY)
        if(CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
            add_compile_options(-fdiagnostics-format=json)
            message(STATUS "Build AI-friendly: Clang JSON diagnostics enabled")
        elseif(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
            # GCC 13+ supports -fdiagnostics-format=json
            add_compile_options(-fdiagnostics-format=json)
            message(STATUS "Build AI-friendly: GCC JSON diagnostics enabled")
        endif()
    endif()
endif()

# ============================================================================
# Extracted Libraries
# ============================================================================
# Box Spread C++ Engine (broker-agnostic library)
# Default to OFF for now - requires TWS adapter implementation (see MONOREPO_REORGANIZATION_PLAN.md)
option(USE_BOX_SPREAD_CPP_LIB "Use extracted box-spread-cpp library" OFF)
if(USE_BOX_SPREAD_CPP_LIB)
    set(BOX_SPREAD_CPP_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../libs/box-spread-cpp")
    if(EXISTS "${BOX_SPREAD_CPP_DIR}/CMakeLists.txt")
        add_subdirectory(${BOX_SPREAD_CPP_DIR} box_spread_cpp)
        message(STATUS "Using extracted box-spread-cpp library from: ${BOX_SPREAD_CPP_DIR}")
        # Add library include directory
        if(TARGET box_spread_cpp)
            get_target_property(BOX_SPREAD_CPP_INCLUDE_DIR box_spread_cpp INTERFACE_INCLUDE_DIRECTORIES)
            if(NOT BOX_SPREAD_CPP_INCLUDE_DIR)
                # Fallback: use the library's include directory
                set(BOX_SPREAD_CPP_INCLUDE_DIR "${BOX_SPREAD_CPP_DIR}/include")
            endif()
            message(STATUS "Box Spread C++ library include directory: ${BOX_SPREAD_CPP_INCLUDE_DIR}")
        endif()
    else()
        message(WARNING "box-spread-cpp library not found at ${BOX_SPREAD_CPP_DIR}")
        message(WARNING "Initialize submodule with: git submodule update --init --recursive")
        set(USE_BOX_SPREAD_CPP_LIB OFF)
    endif()
else()
    message(STATUS "Using local box spread implementation (not extracted library)")
endif()

# ============================================================================
# Dependencies
# ============================================================================
include(FetchContent)

set(NLOHMANN_JSON_URL_BASE "https://github.com/nlohmann/json/releases/download")
set(NLOHMANN_JSON_URL_VERSION "v3.11.3")
set(NLOHMANN_JSON_ARCHIVE "json.tar.xz")
set(NLOHMANN_JSON_URL_SEGMENTS
    "${NLOHMANN_JSON_URL_BASE}"
    "${NLOHMANN_JSON_URL_VERSION}"
    "${NLOHMANN_JSON_ARCHIVE}"
)
string(JOIN "/" NLOHMANN_JSON_URL ${NLOHMANN_JSON_URL_SEGMENTS})
set(NLOHMANN_JSON_SHA256
    "SHA256=d6c65aca6b1ed68e7a182f4757257b107ae403032760ed6ef121c9d55e81757d"
)

fetchcontent_declare(
    nlohmann_json
    URL ${NLOHMANN_JSON_URL}
    URL_HASH ${NLOHMANN_JSON_SHA256}
    DOWNLOAD_EXTRACT_TIMESTAMP TRUE
)

set(SPDLOG_REPOSITORY "https://github.com/gabime/spdlog.git")
# Configure spdlog as header-only to avoid LTO linking issues
set(SPDLOG_BUILD_SHARED OFF CACHE BOOL "" FORCE)
set(SPDLOG_FMT_EXTERNAL OFF CACHE BOOL "" FORCE)
fetchcontent_declare(
    spdlog
    GIT_REPOSITORY ${SPDLOG_REPOSITORY}
    GIT_TAG v1.13.0
    GIT_SHALLOW TRUE
)

set(CLI11_REPOSITORY "https://github.com/CLIUtils/CLI11.git")
set(PATCH_CMAKE_MINIMUM "${CMAKE_SOURCE_DIR}/scripts/patch_cmake_minimum.cmake")
fetchcontent_declare(
    CLI11
    GIT_REPOSITORY ${CLI11_REPOSITORY}
    GIT_TAG v2.4.1
    GIT_SHALLOW TRUE
    PATCH_COMMAND ${CMAKE_COMMAND} -P ${PATCH_CMAKE_MINIMUM}
)

# FTXUI removed - C++ TUI has been replaced by Python TUI
# set(FTXUI_REPOSITORY "https://github.com/ArthurSonzogni/FTXUI.git")
# fetchcontent_declare(
#     ftxui
#     GIT_REPOSITORY ${FTXUI_REPOSITORY}
#     GIT_TAG v5.0.0
#     GIT_SHALLOW TRUE
# )

set(EIGEN3_REPOSITORY "https://gitlab.com/libeigen/eigen.git")
fetchcontent_declare(
    Eigen3
    GIT_REPOSITORY ${EIGEN3_REPOSITORY}
    GIT_TAG master
    GIT_SHALLOW TRUE
    PATCH_COMMAND ${CMAKE_COMMAND} -P ${PATCH_CMAKE_MINIMUM}
)

# Find Boost (required for QuantLib)
# Set Boost root if installed via Homebrew
set(BOOST_FOUND_DIRECTLY FALSE)
# Check both Intel (/usr/local) and Apple Silicon (/opt/homebrew) Homebrew paths
set(BOOST_CELLAR_PATHS "/opt/homebrew/Cellar/boost" "/usr/local/Cellar/boost")
set(BOOST_CELLAR_DIR "")
foreach(BOOST_CELLAR_PATH ${BOOST_CELLAR_PATHS})
    if(EXISTS "${BOOST_CELLAR_PATH}")
        # Find the actual Boost version directory
        file(GLOB BOOST_CELLAR_DIRS "${BOOST_CELLAR_PATH}/*")
        if(BOOST_CELLAR_DIRS)
            list(GET BOOST_CELLAR_DIRS 0 BOOST_CELLAR_DIR)
            break()
        endif()
    endif()
endforeach()

if(BOOST_CELLAR_DIR)
    set(Boost_ROOT "${BOOST_CELLAR_DIR}")
    set(BOOST_LIB_DIR "${BOOST_CELLAR_DIR}/lib")
    set(BOOST_INCLUDE_DIR "${BOOST_CELLAR_DIR}/include")
    message(STATUS "Found Boost in Homebrew Cellar: ${BOOST_CELLAR_DIR}")

        # Try to find libraries directly (check both .dylib and .a)
        find_library(BOOST_DATE_TIME_LIB
            NAMES boost_date_time libboost_date_time
            PATHS ${BOOST_LIB_DIR} /usr/local/lib
            NO_DEFAULT_PATH
        )
        find_library(BOOST_FILESYSTEM_LIB
            NAMES boost_filesystem libboost_filesystem
            PATHS ${BOOST_LIB_DIR} /usr/local/lib
            NO_DEFAULT_PATH
        )
        find_library(BOOST_SYSTEM_LIB
            NAMES boost_system libboost_system
            PATHS ${BOOST_LIB_DIR} /usr/local/lib
            NO_DEFAULT_PATH
        )

        # Debug output
        if(BOOST_DATE_TIME_LIB)
            message(STATUS "Found boost_date_time: ${BOOST_DATE_TIME_LIB}")
        endif()
        if(BOOST_FILESYSTEM_LIB)
            message(STATUS "Found boost_filesystem: ${BOOST_FILESYSTEM_LIB}")
        endif()
        if(BOOST_SYSTEM_LIB)
            message(STATUS "Found boost_system: ${BOOST_SYSTEM_LIB}")
        endif()

        # boost_system might be in /usr/local/lib (symlink) or header-only
        if(NOT BOOST_SYSTEM_LIB)
            find_library(BOOST_SYSTEM_LIB
                NAMES boost_system libboost_system
                PATHS /usr/local/lib ${BOOST_LIB_DIR}
                NO_DEFAULT_PATH
            )
        endif()

        # Proceed if we have at least date_time and filesystem
        # boost_system might be header-only in newer Boost versions
        if(BOOST_DATE_TIME_LIB AND BOOST_FILESYSTEM_LIB)
            set(BOOST_FOUND_DIRECTLY TRUE)
            set(Boost_LIBRARIES "${BOOST_DATE_TIME_LIB}" "${BOOST_FILESYSTEM_LIB}")
            if(BOOST_SYSTEM_LIB)
                set(Boost_LIBRARIES "${BOOST_DATE_TIME_LIB}" "${BOOST_FILESYSTEM_LIB}" "${BOOST_SYSTEM_LIB}")
            endif()
            set(Boost_INCLUDE_DIRS "${BOOST_INCLUDE_DIR}")
            set(Boost_FOUND TRUE)
            message(STATUS "Found Boost libraries directly")
        endif()
endif()

# Try find_package if direct finding didn't work
if(NOT BOOST_FOUND_DIRECTLY)
    set(Boost_NO_BOOST_CMAKE ON)
    # Suppress "New Boost version may have incorrect or missing dependencies" when
    # system Boost is newer than FindBoost's hardcoded list (e.g. Boost 1.88 on CMake 3.31).
    set(Boost_NO_WARN_NEW_VERSIONS ON)
    find_package(Boost QUIET COMPONENTS date_time filesystem system)
    if(NOT Boost_FOUND)
        message(FATAL_ERROR "Could not find Boost. Please install via: brew install boost")
    endif()
endif()

# QuantLib via FetchContent
set(QUANTLIB_REPOSITORY "https://github.com/lballabio/QuantLib.git")
# Reduce build and optional deps when used as subproject
set(QL_BUILD_EXAMPLES OFF CACHE BOOL "Build QuantLib examples" FORCE)
set(QL_BUILD_TEST_SUITE OFF CACHE BOOL "Build QuantLib test suite" FORCE)
fetchcontent_declare(
    QuantLib
    GIT_REPOSITORY ${QUANTLIB_REPOSITORY}
    GIT_TAG v1.41
    GIT_SHALLOW TRUE
)

# NLopt via FetchContent
set(NLOPT_REPOSITORY "https://github.com/stevengj/nlopt.git")
fetchcontent_declare(
    NLopt
    GIT_REPOSITORY ${NLOPT_REPOSITORY}
    GIT_TAG v2.9.1
    GIT_SHALLOW TRUE
    PATCH_COMMAND patch -p1 -i "${CMAKE_CURRENT_SOURCE_DIR}/cmake/nlopt-enable-cxx.patch"
)
set(NLOPT_LICENSE_MIT ON CACHE BOOL "Use MIT license instead of LGPL")

# NATS C client library (optional, for message queue integration)
option(ENABLE_NATS "Enable NATS message queue integration" OFF)
if(ENABLE_NATS)
    set(NATS_C_REPOSITORY "https://github.com/nats-io/nats.c.git")
    fetchcontent_declare(
        nats_c
        GIT_REPOSITORY ${NATS_C_REPOSITORY}
        GIT_TAG v3.8.0
        GIT_SHALLOW TRUE
    )
    message(STATUS "NATS integration enabled - will fetch nats.c library")
endif()

# Ensure macOS SDK C++ headers are in CXX flags for FetchContent subprojects (e.g. NLopt try_compile)
if(APPLE)
    execute_process(COMMAND xcrun --show-sdk-path OUTPUT_VARIABLE _NLOPT_SDK OUTPUT_STRIP_TRAILING_WHITESPACE ERROR_QUIET)
    if(_NLOPT_SDK)
        set(_NLOPT_SDK_CXX "${_NLOPT_SDK}/usr/include/c++/v1")
        if(EXISTS "${_NLOPT_SDK_CXX}/string")
            set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -stdlib++-isystem${_NLOPT_SDK_CXX}" CACHE STRING "" FORCE)
            message(STATUS "NLopt/FetchContent: using SDK C++ headers ${_NLOPT_SDK_CXX}")
        endif()
    endif()
endif()

fetchcontent_makeavailable(nlohmann_json spdlog CLI11 Eigen3 QuantLib NLopt)

# QuantLib when built via FetchContent defines target "ql_library" in ql/CMakeLists.txt
# (EXPORT_NAME QuantLib); the namespace alias QuantLib::QuantLib is only created at install.
# Our global add_compile_options(-Wunused etc.) apply to subprojects; suppress unused-parameter
# (and similar) for the QuantLib target so third-party code does not fail the build.
set(_ql_target "")
if(TARGET ql_library)
  set(_ql_target ql_library)
elseif(TARGET ql)
  set(_ql_target ql)
elseif(TARGET QuantLib)
  set(_ql_target QuantLib)
endif()
if(_ql_target AND CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
  target_compile_options(${_ql_target} PRIVATE
    -Wno-unused-parameter
    -Wno-unused-variable
    -Wno-shadow
    -Wno-conversion
    -Wno-sign-conversion
    -Wno-old-style-cast
    -Wno-overloaded-virtual
  )
  message(STATUS "QuantLib: suppressed third-party warnings for ${_ql_target}")
endif()

if(NOT TARGET QuantLib::QuantLib)
  if(TARGET ql_library)
    add_library(QuantLib::QuantLib ALIAS ql_library)
    message(STATUS "QuantLib: using alias QuantLib::QuantLib -> ql_library")
  elseif(TARGET ql)
    add_library(QuantLib::QuantLib ALIAS ql)
    message(STATUS "QuantLib: using alias QuantLib::QuantLib -> ql")
  elseif(TARGET QuantLib)
    add_library(QuantLib::QuantLib ALIAS QuantLib)
    message(STATUS "QuantLib: using alias QuantLib::QuantLib -> QuantLib")
  else()
    message(FATAL_ERROR "QuantLib FetchContent did not define target ql_library, ql, or QuantLib")
  endif()
endif()

# NLopt when built via FetchContent defines target "nlopt"; NLopt::nlopt is only created at install.
if(NOT TARGET NLopt::nlopt)
  if(TARGET nlopt)
    add_library(NLopt::nlopt ALIAS nlopt)
    message(STATUS "NLopt: using alias NLopt::nlopt -> nlopt")
  else()
    message(FATAL_ERROR "NLopt FetchContent did not define target nlopt")
  endif()
endif()

if(BUILD_TESTING)
    set(CATCH2_REPOSITORY "https://github.com/catchorg/Catch2.git")
    fetchcontent_declare(
        Catch2
        GIT_REPOSITORY ${CATCH2_REPOSITORY}
        GIT_TAG v3.5.4
        GIT_SHALLOW TRUE
    )
    fetchcontent_makeavailable(Catch2)
    include(CTest)
    enable_testing()
endif()

# ====================================================================================
# Protobuf (generated from proto/messages.proto; regenerated at build if needed)
# ============================================================================
set(PROTO_GENERATED_DIR "${CMAKE_CURRENT_SOURCE_DIR}/generated")
set(_platform_proto_src "${CMAKE_SOURCE_DIR}/proto/messages.proto")
find_package(Protobuf QUIET)
if(Protobuf_FOUND)
    set(_platform_pb_cc "${PROTO_GENERATED_DIR}/messages.pb.cc")
    set(_platform_pb_h  "${PROTO_GENERATED_DIR}/messages.pb.h")
    add_custom_command(
        OUTPUT "${_platform_pb_cc}" "${_platform_pb_h}"
        COMMAND ${CMAKE_COMMAND} -E make_directory "${PROTO_GENERATED_DIR}"
        COMMAND ${Protobuf_PROTOC_EXECUTABLE}
            -I "${CMAKE_SOURCE_DIR}/proto"
            --cpp_out="${PROTO_GENERATED_DIR}"
            "${_platform_proto_src}"
        DEPENDS "${_platform_proto_src}"
        COMMENT "Generating C++ protobuf from proto/messages.proto"
    )
    add_custom_target(generate_platform_proto
        DEPENDS "${_platform_pb_cc}" "${_platform_pb_h}"
    )
    add_library(platform_proto STATIC "${_platform_pb_cc}")
    add_dependencies(platform_proto generate_platform_proto)
    target_include_directories(platform_proto PUBLIC "${PROTO_GENERATED_DIR}")
    target_link_libraries(platform_proto PUBLIC protobuf::libprotobuf)
    message(STATUS "Protobuf: C++ codegen from proto/messages.proto (run at build)")
else()
    if(EXISTS "${PROTO_GENERATED_DIR}/messages.pb.cc")
        message(WARNING "Generated proto found but Protobuf not found. Install: brew install protobuf")
    else()
        message(STATUS "Protobuf not found; platform_proto disabled. Install: brew install protobuf")
    endif()
endif()

# Proto adapter (types <-> proto at boundaries); only when platform_proto exists
if(TARGET platform_proto)
    add_library(proto_adapter STATIC src/proto_adapter.cpp)
    target_include_directories(proto_adapter
        PUBLIC
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${PROTO_GENERATED_DIR}
    )
    target_link_libraries(proto_adapter PUBLIC platform_proto nlohmann_json::nlohmann_json)
    message(STATUS "Proto adapter library: proto_adapter")

    # Standalone test for proto adapter (no TWS/Intel deps)
    if(BUILD_TESTING)
        add_executable(proto_adapter_tests tests/test_proto_adapter.cpp)
        target_include_directories(proto_adapter_tests PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${PROTO_GENERATED_DIR}
        )
        target_link_libraries(proto_adapter_tests PRIVATE
            proto_adapter
            Catch2::Catch2WithMain
        )
        if(NOT ABSEIL_LIBS)
            file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.dylib")
        endif()
        if(ABSEIL_LIBS)
            target_link_libraries(proto_adapter_tests PRIVATE ${ABSEIL_LIBS})
        endif()
        target_sources(proto_adapter_tests PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/tests/types_stub_for_proto_test.cpp
        )
        include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)
        add_test(NAME proto_adapter_tests COMMAND $<TARGET_FILE:proto_adapter_tests>)
    endif()
endif()

# ====================================================================================
# TWS API (IBKR C++ API)
# Prefer GitHub repo layout (source/cppclient/client) via TWS_API_SOURCE_DIR,
# e.g. -DTWS_API_SOURCE_DIR=/path/to/tws-api or ../tws-api. Falls back to
# vendored zip layout (third_party/tws-api/IBJts/source/cppclient).
# ============================================================================
get_filename_component(_tws_api_default_source_dir "${CMAKE_CURRENT_SOURCE_DIR}/../tws-api" REALPATH)
get_filename_component(_tws_api_repo_sibling_source_dir "${CMAKE_SOURCE_DIR}/../tws-api" REALPATH)
set(TWS_API_SOURCE_DIR "" CACHE PATH "Root of tws-api repo (GitHub layout: source/cppclient/client). Example: ../tws-api")
if(NOT TWS_API_SOURCE_DIR AND EXISTS "${_tws_api_default_source_dir}/source/cppclient/client")
  set(TWS_API_SOURCE_DIR "${_tws_api_default_source_dir}" CACHE PATH "Root of tws-api repo (GitHub layout). Auto-detected ../tws-api." FORCE)
endif()
if(NOT TWS_API_SOURCE_DIR AND EXISTS "${_tws_api_repo_sibling_source_dir}/source/cppclient/client")
  set(TWS_API_SOURCE_DIR "${_tws_api_repo_sibling_source_dir}" CACHE PATH "Root of tws-api repo (GitHub layout). Auto-detected repo-sibling ../tws-api." FORCE)
endif()

set(_tws_api_use_github_dir OFF)
if(TWS_API_SOURCE_DIR)
  _expand_user_path(TWS_API_SOURCE_DIR _resolved_tws_api_source_dir)
  if(_resolved_tws_api_source_dir AND EXISTS "${_resolved_tws_api_source_dir}/source/cppclient/client")
    set(_tws_api_use_github_dir ON)
    set(TWS_API_VENDOR_ROOT "${_resolved_tws_api_source_dir}/source/cppclient")
    set(TWS_API_INCLUDE_DIR "${TWS_API_VENDOR_ROOT}/client" CACHE PATH "Path to TWS API include directory" FORCE)
    message(STATUS "Using TWS API from GitHub layout: ${_resolved_tws_api_source_dir}")
  else()
    message(WARNING "TWS_API_SOURCE_DIR set but source/cppclient/client not found: ${TWS_API_SOURCE_DIR}")
  endif()
endif()

if(NOT _tws_api_use_github_dir)
set(TWS_API_VENDOR_ROOT "${CMAKE_CURRENT_SOURCE_DIR}")
cmake_path(APPEND TWS_API_VENDOR_ROOT
    "third_party" "tws-api" "source" "cppclient"
)

set(DEFAULT_TWS_API_INCLUDE_DIR "${TWS_API_VENDOR_ROOT}")
cmake_path(APPEND DEFAULT_TWS_API_INCLUDE_DIR "client")
set(
    TWS_API_INCLUDE_DIR
    "${DEFAULT_TWS_API_INCLUDE_DIR}"
    CACHE PATH "Path to TWS API include directory"
)
# Vendored GitHub layout: third_party/tws-api/source/cppclient/client exists; enable build from source
set(_tws_vendor_client "${TWS_API_VENDOR_ROOT}/client")
if(EXISTS "${_tws_vendor_client}/Contract.h")
  set(_tws_api_use_github_dir ON)
  message(STATUS "Using TWS API from vendored GitHub layout: ${TWS_API_VENDOR_ROOT}")
endif()
endif()

if(DEFINED IBAPI_INCLUDE_DIR AND NOT IBAPI_INCLUDE_DIR STREQUAL "")
    _expand_user_path(IBAPI_INCLUDE_DIR _resolved_ibapi_include_dir)
    if(_resolved_ibapi_include_dir AND EXISTS "${_resolved_ibapi_include_dir}")
        set(
            TWS_API_INCLUDE_DIR
            "${_resolved_ibapi_include_dir}"
            CACHE PATH "Path to TWS API include directory"
            FORCE
        )
        message(STATUS "TWS API include directory supplied via IBAPI_INCLUDE_DIR: ${TWS_API_INCLUDE_DIR}")
    else()
        message(WARNING "IBAPI_INCLUDE_DIR was provided but does not exist: ${IBAPI_INCLUDE_DIR}")
    endif()
endif()

# Repair stale cached include paths by preferring a valid adjacent tws-api checkout
# when the cached vendor path no longer contains real TWS headers.
set(_tws_api_recovery_source_dir "")
if(EXISTS "${_tws_api_default_source_dir}/source/cppclient/client/Contract.h")
    set(_tws_api_recovery_source_dir "${_tws_api_default_source_dir}")
elseif(EXISTS "${_tws_api_repo_sibling_source_dir}/source/cppclient/client/Contract.h")
    set(_tws_api_recovery_source_dir "${_tws_api_repo_sibling_source_dir}")
endif()

if((NOT EXISTS "${TWS_API_INCLUDE_DIR}/Contract.h") AND _tws_api_recovery_source_dir)
    set(TWS_API_SOURCE_DIR "${_tws_api_recovery_source_dir}" CACHE PATH "Root of tws-api repo (GitHub layout)." FORCE)
    set(TWS_API_VENDOR_ROOT "${_tws_api_recovery_source_dir}/source/cppclient")
    set(
        TWS_API_INCLUDE_DIR
        "${TWS_API_VENDOR_ROOT}/client"
        CACHE PATH "Path to TWS API include directory"
        FORCE
    )
    set(_tws_api_use_github_dir ON)
    message(STATUS "Recovered stale TWS API include cache via adjacent tws-api checkout: ${TWS_API_INCLUDE_DIR}")
endif()

if(EXISTS "${TWS_API_INCLUDE_DIR}/Contract.h")
    message(STATUS "Using TWS API include dir: ${TWS_API_INCLUDE_DIR}")
else()
    message(WARNING "TWS API headers not found at ${TWS_API_INCLUDE_DIR}")
    message(WARNING "Download from: https://interactivebrokers.github.io/")
    message(WARNING "Extract to: ${CMAKE_CURRENT_SOURCE_DIR}/third_party/tws-api/")
    message(WARNING "Or use GitHub repo: -DTWS_API_SOURCE_DIR=/path/to/tws-api")
endif()

# Determine library extension based on platform
if(APPLE)
    set(_twsapi_lib_ext ".dylib")
elseif(UNIX)
    set(_twsapi_lib_ext ".so")
else()
    set(_twsapi_lib_ext ".dll")
endif()

if(NOT _tws_api_use_github_dir)
set(DEFAULT_TWS_API_LIB "${TWS_API_VENDOR_ROOT}")
cmake_path(APPEND DEFAULT_TWS_API_LIB "client" "build" "lib" "libtwsapi${_twsapi_lib_ext}")
else()
set(DEFAULT_TWS_API_LIB "")
endif()
set(
    TWS_API_LIB
    "${DEFAULT_TWS_API_LIB}"
    CACHE FILEPATH "Path to the TWS API shared library"
)

set(_ibapi_lib_candidates)
if(DEFINED IBAPI_LIB AND NOT IBAPI_LIB STREQUAL "")
    _expand_user_path(IBAPI_LIB _resolved_ibapi_lib)
    if(_resolved_ibapi_lib)
        list(APPEND _ibapi_lib_candidates "${_resolved_ibapi_lib}")
    endif()
endif()

if(EXISTS "${TWS_API_INCLUDE_DIR}")
    set(_candidate "${TWS_API_INCLUDE_DIR}")
    cmake_path(APPEND _candidate "build" "lib" "libtwsapi${_twsapi_lib_ext}")
    list(APPEND _ibapi_lib_candidates "${_candidate}")

    set(_candidate "${TWS_API_INCLUDE_DIR}")
    cmake_path(APPEND _candidate "lib" "libtwsapi${_twsapi_lib_ext}")
    list(APPEND _ibapi_lib_candidates "${_candidate}")

    if(APPLE)
        set(_candidate "${TWS_API_INCLUDE_DIR}")
        cmake_path(APPEND _candidate "libTwsApiCpp.dylib")
        list(APPEND _ibapi_lib_candidates "${_candidate}")
    endif()
endif()

set(_candidate "${DEFAULT_TWS_API_LIB}")
list(APPEND _ibapi_lib_candidates "${_candidate}")

if(APPLE)
    set(_candidate "${TWS_API_VENDOR_ROOT}")
    cmake_path(APPEND _candidate "libTwsApiCpp.dylib")
    list(APPEND _ibapi_lib_candidates "${_candidate}")
endif()

set(_candidate "${TWS_API_VENDOR_ROOT}")
cmake_path(APPEND _candidate "lib" "libtwsapi${_twsapi_lib_ext}")
list(APPEND _ibapi_lib_candidates "${_candidate}")

# When using GitHub layout there is no prebuilt lib; build from source.
if(_tws_api_use_github_dir AND NOT TWS_API_BUILD_VENDOR)
  set(TWS_API_BUILD_VENDOR ON CACHE BOOL "Build TWS API library from source (ibapi_cmake)" FORCE)
  message(STATUS "TWS_API_BUILD_VENDOR enabled (required for GitHub TWS API layout)")
endif()

if(TWS_API_BUILD_VENDOR)
    # Discover Protobuf so we can pass Protobuf_ROOT to the TWS API build (ibapi_cmake).
    # Avoids "No rule to make target '/usr/local/lib/libprotobuf.dylib'" when lib is elsewhere (e.g. Homebrew).
    set(Protobuf_ROOT "" CACHE PATH "Root of Protocol Buffers install (include + lib)")
    if(NOT Protobuf_ROOT)
        find_library(_protobuf_lib NAMES protobuf libprotobuf
            PATHS /opt/homebrew/lib /usr/local/lib /usr/lib
            NO_DEFAULT_PATH
        )
        if(_protobuf_lib)
            get_filename_component(Protobuf_ROOT "${_protobuf_lib}" DIRECTORY)
            get_filename_component(Protobuf_ROOT "${Protobuf_ROOT}" DIRECTORY)
            message(STATUS "Found Protobuf for TWS API build: ${Protobuf_ROOT}")
        endif()
    endif()

    # Build TWS API using our ibapi_cmake (finds protobuf/Abseil; vendored client CMakeLists has hardcoded paths).
    if(_tws_api_use_github_dir)
        set(_tws_api_client_dir "${TWS_API_VENDOR_ROOT}/client")
    elseif(NOT EXISTS "${TWS_API_VENDOR_ROOT}/../../CMakeLists.txt")
        message(FATAL_ERROR "Vendored TWS API source not found at ${TWS_API_VENDOR_ROOT}/../../CMakeLists.txt. Use -DTWS_API_SOURCE_DIR=/path/to/tws-api for GitHub layout.")
    endif()
    if(NOT EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/ibapi_cmake/CMakeLists.txt")
        message(FATAL_ERROR "ibapi_cmake not found; required for TWS_API_BUILD_VENDOR")
    endif()
    include(ExternalProject)
    set(TWS_API_EXTERNAL_BINARY_DIR "${CMAKE_BINARY_DIR}/tws_api_vendor_build")
    set(_twsapi_cmake_args
        -DCMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE}
        -DCMAKE_INSTALL_PREFIX=${TWS_API_EXTERNAL_BINARY_DIR}/install
        -DCMAKE_POLICY_DEFAULT_CMP0126=NEW
    )
    # Pass SDK and C++ header fix to TWS API sub-build (ExternalProject does not inherit CMAKE_CXX_FLAGS).
    # ibapi_cmake adds the SDK C++ include dir when CMAKE_OSX_SYSROOT is set.
    if(APPLE)
        set(_tws_sysroot "${CMAKE_OSX_SYSROOT}")
        if(NOT _tws_sysroot)
            execute_process(
                COMMAND xcrun --show-sdk-path
                OUTPUT_VARIABLE _tws_sysroot
                OUTPUT_STRIP_TRAILING_WHITESPACE
                ERROR_QUIET
            )
        endif()
        if(_tws_sysroot)
            list(APPEND _twsapi_cmake_args -DCMAKE_OSX_SYSROOT=${_tws_sysroot})
            list(APPEND _twsapi_cmake_args "-DCMAKE_CXX_FLAGS=${CMAKE_CXX_FLAGS}")
        endif()
    endif()
    if(Protobuf_ROOT)
        list(APPEND _twsapi_cmake_args -DProtobuf_ROOT=${Protobuf_ROOT})
    endif()
    if(_tws_api_use_github_dir)
        list(APPEND _twsapi_cmake_args -DTWS_API_CLIENT_DIR=${_tws_api_client_dir})
    endif()
    if(APPLE)
        set(_twsapi_lib_filename "libtwsapi.dylib")
        if(CMAKE_BUILD_TYPE STREQUAL "Debug")
            set(_twsapi_lib_filename "libtwsapid.dylib")
        endif()
    else()
        set(_twsapi_lib_filename "libtwsapi.so")
        if(CMAKE_BUILD_TYPE STREQUAL "Debug")
            set(_twsapi_lib_filename "libtwsapid.so")
        endif()
    endif()
    set(_twsapi_lib_path "${TWS_API_EXTERNAL_BINARY_DIR}/lib/${_twsapi_lib_filename}")
    ExternalProject_Add(twsapi_external
        SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/ibapi_cmake"
        BINARY_DIR "${TWS_API_EXTERNAL_BINARY_DIR}"
        CMAKE_ARGS ${_twsapi_cmake_args}
        INSTALL_COMMAND ""
        BUILD_BYPRODUCTS "${_twsapi_lib_path}"
    )
    set(
        TWS_API_LIB
        "${_twsapi_lib_path}"
        CACHE FILEPATH "Path to the TWS API shared library"
        FORCE
    )
    list(APPEND _ibapi_lib_candidates "${TWS_API_LIB}")
endif()

if(NOT TWS_API_LIB OR NOT EXISTS "${TWS_API_LIB}")
    foreach(_candidate IN LISTS _ibapi_lib_candidates)
        if(_candidate AND EXISTS "${_candidate}")
            set(
                TWS_API_LIB
                "${_candidate}"
                CACHE FILEPATH "Path to the TWS API shared library"
                FORCE
            )
            message(STATUS "Resolved TWS API library: ${TWS_API_LIB}")
            break()
        endif()
    endforeach()
endif()

set(_intel_u2 "${CMAKE_CURRENT_SOURCE_DIR}/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a")
set(_intel_u4 "${CMAKE_CURRENT_SOURCE_DIR}/third_party/IntelRDFPMathLib20U4/LIBRARY/libbid.a")
# U4 is populated by ./scripts/fetch_third_party.sh; prefer it over legacy U2.
if(EXISTS "${_intel_u4}")
    set(INTEL_DECIMAL_LIB "${_intel_u4}")
else()
    set(INTEL_DECIMAL_LIB "${_intel_u2}")
endif()
unset(_intel_u2)
unset(_intel_u4)

set(_missing_native_deps "")
set(_native_prereqs_met TRUE)

if(NOT TWS_API_LIB OR NOT EXISTS "${TWS_API_LIB}")
    if(NOT TWS_API_BUILD_VENDOR)
        list(APPEND _missing_native_deps "TWS API shared library")
        set(_native_prereqs_met FALSE)
    endif()
endif()

if(NOT EXISTS "${INTEL_DECIMAL_LIB}")
    list(APPEND _missing_native_deps "Intel decimal math library (libbid.a)")
    set(_native_prereqs_met FALSE)
endif()

if(ENABLE_NATIVE_CLI AND NOT _native_prereqs_met)
    string(REPLACE ";" ", " _missing_native_deps_str "${_missing_native_deps}")
    message(FATAL_ERROR
        "Native CLI requires: ${_missing_native_deps_str}.\n"
        "Run from repo root: ./scripts/fetch_third_party.sh\n"
        "Then reconfigure. See README for TWS API and Intel decimal setup.")
endif()

set(_build_native_cli OFF)
if(ENABLE_NATIVE_CLI AND _native_prereqs_met)
    set(_build_native_cli ON)
endif()

# ====================================================================================
# Main Executable
# ============================================================================
if(_build_native_cli)
    set(SOURCES
        src/ib_box_spread.cpp
        src/tws_client.cpp
        src/tws_client_impl_ewrapper.cpp
        src/tws_conversions.cpp
        src/tws_connection.cpp
        src/tws_market_data.cpp
        src/tws_orders.cpp
        src/tws_positions.cpp
        src/tws_contracts.cpp
        src/types.cpp
        src/connection_utils.cpp
        src/tws_error_codes.cpp
        src/market_hours.cpp
        src/greeks_calculator.cpp
        src/rate_limiter.cpp
        src/path_validator.cpp
        src/config_manager.cpp
        src/nats_client.cpp
        src/pcap_capture.cpp
        src/convexity_calculator.cpp
        src/financing_optimizer.cpp
        src/loan_position.cpp
        src/margin_calculator.cpp
        src/cache_client.cpp
        src/asset_relationship.cpp
        src/collateral_valuator.cpp
        src/financing_instrument_registry.cpp
    )

    set(HEADERS
        include/tws_client.h
        include/tws_conversions.h
        include/tws_connection.h
        include/tws_market_data.h
        include/tws_orders.h
        include/tws_positions.h
        include/tws_contracts.h
        include/connection_utils.h
        include/tws_error_codes.h
        include/rate_limiter.h
        include/market_hours.h
        include/greeks_calculator.h
        include/path_validator.h
        include/config_manager.h
        include/pcap_capture.h
        include/convexity_calculator.h
        include/financing_optimizer.h
        include/types.h
        include/loan_position.h
        include/loan_manager.h
        include/margin_calculator.h
        include/cache_client.h
        include/asset_relationship.h
        include/collateral_valuator.h
        include/financing_instrument.h
    )

    # Conditionally include local source files only if NOT using extracted library
    if(NOT USE_BOX_SPREAD_CPP_LIB)
        list(APPEND SOURCES
            src/option_chain.cpp
            src/strategies/box_spread/box_spread_strategy.cpp
            src/strategies/box_spread/box_spread_bag.cpp
            src/order_manager.cpp
            src/risk_calculator.cpp
            src/risk_calculator_var.cpp
            src/risk_calculator_sizing.cpp
            src/risk_calculator_stats.cpp
            src/greeks_calculator.cpp
        )
        list(APPEND HEADERS
            include/option_chain.h
            include/strategies/box_spread/box_spread_strategy.h
            include/strategies/box_spread/box_spread_bag.h
            include/order_manager.h
            include/greeks_calculator.h
            include/risk_calculator.h
        )
        message(STATUS "Including local box spread implementation files")
    else()
        # When using extracted library, include broker adapters
        list(APPEND SOURCES
            src/brokers/tws_adapter.cpp
            src/brokers/http_client.cpp
            src/brokers/alpaca_adapter.cpp
        )
        list(APPEND HEADERS
            include/brokers/tws_adapter.h
            include/brokers/http_client.h
            include/brokers/alpaca_adapter.h
        )
        message(STATUS "Excluding local box spread files (using extracted library with broker adapters)")
    endif()

    # Version suffix (empty for release, "-prerelease" for development)
    # Set via: cmake -DVERSION_SUFFIX="" for release, or omit for prerelease
    if(NOT DEFINED VERSION_SUFFIX)
        set(VERSION_SUFFIX "-prerelease")
    endif()

    # Generate version.h from CMake PROJECT_VERSION
    configure_file(
        "${CMAKE_CURRENT_SOURCE_DIR}/include/version.h.in"
        "${CMAKE_CURRENT_BINARY_DIR}/include/version.h"
        @ONLY
    )

    add_executable(ib_box_spread ${SOURCES} ${HEADERS})

    # Suppress -Wdangling-reference from spdlog/fmt when formatting std::string (false positives)
    if(CMAKE_CXX_COMPILER_ID MATCHES "Clang|GNU")
        target_compile_options(ib_box_spread PRIVATE -Wno-dangling-reference)
    endif()

    # Include the generated version header in the build
    # When TWS_API_BUILD_VENDOR: put proto_generated FIRST so regenerated .pb.h (matching system
    # protobuf) are used instead of vendored 6.33.4 .pb.h in TWS_API_INCLUDE_DIR (see
    # docs/BUILD_FAILURES_AND_DEPENDENCIES.md § Protobuf / TWS API version alignment).
    target_include_directories(ib_box_spread
        PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${CMAKE_CURRENT_BINARY_DIR}/include  # For generated version.h
    )
    if(TWS_API_BUILD_VENDOR)
        target_include_directories(ib_box_spread PRIVATE "${TWS_API_EXTERNAL_BINARY_DIR}/proto_generated")
    endif()
    target_include_directories(ib_box_spread PRIVATE ${TWS_API_INCLUDE_DIR})
    if(USE_BOX_SPREAD_CPP_LIB AND DEFINED BOX_SPREAD_CPP_INCLUDE_DIR)
        target_include_directories(ib_box_spread
            PRIVATE
                ${BOX_SPREAD_CPP_INCLUDE_DIR}
        )
        target_compile_definitions(ib_box_spread PRIVATE USE_BOX_SPREAD_CPP_LIB)
        message(STATUS "Added box-spread-cpp include directory to ib_box_spread target")
        message(STATUS "Defined USE_BOX_SPREAD_CPP_LIB preprocessor macro")
    endif()

    if(TWS_API_BUILD_VENDOR)
        add_dependencies(ib_box_spread twsapi_external)
    endif()

    # Find libcurl for HTTP client support
    find_package(CURL REQUIRED)

    # Link NATS library if enabled
    if(ENABLE_NATS)
        fetchcontent_makeavailable(nats_c)
        if(TARGET nats)
            target_link_libraries(ib_box_spread PRIVATE nats)
            target_include_directories(ib_box_spread PRIVATE "${CMAKE_BINARY_DIR}/_deps/nats_c-src/src")
            target_compile_definitions(ib_box_spread PRIVATE ENABLE_NATS)
            message(STATUS "Linked NATS library to ib_box_spread")
            # C++ NATS client publishes protobuf (NatsEnvelope); require platform_proto
            if(NOT TARGET platform_proto)
                message(FATAL_ERROR "ENABLE_NATS requires Protobuf for C++ NATS client (protobuf publishing). "
                    "Install protobuf (e.g. brew install protobuf) and reconfigure.")
            endif()
        else()
            message(WARNING "NATS enabled but nats target not found")
        endif()
    endif()

    target_link_libraries(ib_box_spread
        PRIVATE
            nlohmann_json::nlohmann_json
            spdlog::spdlog
            CLI11::CLI11
            Eigen3::Eigen
            QuantLib::QuantLib
            ${Boost_LIBRARIES}
            NLopt::nlopt
            CURL::libcurl
            ${TWS_API_LIB}
            ${INTEL_DECIMAL_LIB}
    )

    # Verify protobuf linkage for proto EWrapper callbacks (tws_client.cpp).
    # libprotobuf is transitively linked via ${TWS_API_LIB}; explicit linkage
    # is a safety net for linkers that do not resolve transitive shared-lib deps.
    find_package(Protobuf QUIET)
    if(Protobuf_FOUND AND TARGET protobuf::libprotobuf)
        target_link_libraries(ib_box_spread PRIVATE protobuf::libprotobuf)
        message(STATUS "Explicitly linked protobuf::libprotobuf to ib_box_spread")
    else()
        message(STATUS "protobuf::libprotobuf target not found; relying on transitive linkage via TWS_API_LIB")
    endif()

    # Expose platform proto generated headers to ib_box_spread (for nats_client.cpp protobuf publishing)
    if(TARGET platform_proto)
        target_include_directories(ib_box_spread PRIVATE "${PROTO_GENERATED_DIR}")
        target_link_libraries(ib_box_spread PRIVATE platform_proto)
        target_compile_definitions(ib_box_spread PRIVATE ENABLE_PROTO)
        message(STATUS "Linked platform_proto to ib_box_spread (ENABLE_PROTO defined)")
    endif()

    # Link to extracted box-spread-cpp library if enabled
    if(USE_BOX_SPREAD_CPP_LIB AND TARGET box_spread_cpp)
        target_link_libraries(ib_box_spread PRIVATE box_spread_cpp)
        message(STATUS "Linked ib_box_spread to box_spread_cpp library")
    endif()

    if(APPLE)
        target_link_libraries(ib_box_spread PRIVATE "-framework CoreFoundation")
        find_library(PTHREAD_LIBRARY pthread)
        if(PTHREAD_LIBRARY)
            target_link_libraries(ib_box_spread PRIVATE ${PTHREAD_LIBRARY})
        endif()
    elseif(UNIX)
        target_link_libraries(ib_box_spread PRIVATE pthread)
        find_library(UUID_LIBRARY uuid)
        if(UUID_LIBRARY)
            target_link_libraries(ib_box_spread PRIVATE ${UUID_LIBRARY})
        endif()
    endif()

    install(TARGETS ib_box_spread
        RUNTIME DESTINATION bin
    )

    install(FILES ${CMAKE_SOURCE_DIR}/README.md
        DESTINATION share/doc/ib_box_spread
    )

    install(DIRECTORY ${CMAKE_SOURCE_DIR}/config/
        DESTINATION share/ib_box_spread/config
        FILES_MATCHING PATTERN "*.example.json"
    )
else()
    message(STATUS "ib_box_spread native CLI target disabled.")
endif()

# ====================================================================================
# TUI Executable (C++) - REMOVED
# ============================================================================
# The C++ TUI has been replaced by the Python TUI (python/tui/).
# See docs/research/architecture/TUI_PYTHON_MIGRATION.md for details.
#
# The Python TUI provides:
# - Same functionality with better maintainability
# - Shared data models with PWA frontend
# - Direct access to Python APIs (IBKR, ORATS, etc.)
# - Modern Textual framework for terminal UI
#
# To run the TUI: python -m python.tui

# ============================================================================
# TWS Connection Test Program
# ============================================================================
if(_build_native_cli AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/tests/test_tws_connection.cpp")
    add_executable(test_tws_connection ${CMAKE_CURRENT_SOURCE_DIR}/tests/test_tws_connection.cpp)

    target_include_directories(test_tws_connection
        PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${CMAKE_CURRENT_SOURCE_DIR}/src
            ${TWS_API_INCLUDE_DIR}
    )
    if(TWS_API_BUILD_VENDOR)
        target_include_directories(test_tws_connection PRIVATE "${TWS_API_EXTERNAL_BINARY_DIR}/proto_generated")
        add_dependencies(test_tws_connection twsapi_external)
    endif()

    target_link_libraries(test_tws_connection
        PRIVATE
            nlohmann_json::nlohmann_json
            spdlog::spdlog
            ${TWS_API_LIB}
            ${INTEL_DECIMAL_LIB}
    )

    # Link with TWS client source files
    target_sources(test_tws_connection PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client_impl_ewrapper.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_conversions.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/types.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/connection_utils.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_error_codes.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/config_manager.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/rate_limiter.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/pcap_capture.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/market_hours.cpp
    )

    if(APPLE)
        target_link_libraries(test_tws_connection PRIVATE "-framework CoreFoundation")
        find_library(PTHREAD_LIBRARY pthread)
        if(PTHREAD_LIBRARY)
            target_link_libraries(test_tws_connection PRIVATE ${PTHREAD_LIBRARY})
        endif()
    elseif(UNIX)
        target_link_libraries(test_tws_connection PRIVATE pthread)
    endif()

    message(STATUS "TWS connection test program: test_tws_connection")
endif()

# ============================================================================
# Position Retrieval Test Program
# ============================================================================
if(_build_native_cli AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/tests/test_positions_live.cpp")
    add_executable(test_positions_live ${CMAKE_CURRENT_SOURCE_DIR}/tests/test_positions_live.cpp)

    target_include_directories(test_positions_live
        PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${CMAKE_CURRENT_SOURCE_DIR}/src
            ${TWS_API_INCLUDE_DIR}
    )
    if(TWS_API_BUILD_VENDOR)
        target_include_directories(test_positions_live PRIVATE "${TWS_API_EXTERNAL_BINARY_DIR}/proto_generated")
        add_dependencies(test_positions_live twsapi_external)
    endif()

    target_link_libraries(test_positions_live
        PRIVATE
            nlohmann_json::nlohmann_json
            spdlog::spdlog
            ${TWS_API_LIB}
            ${INTEL_DECIMAL_LIB}
    )

    # Link with TWS client source files
    target_sources(test_positions_live PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client_impl_ewrapper.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/config_manager.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/rate_limiter.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/market_hours.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/pcap_capture.cpp
    )

    if(APPLE)
        target_link_libraries(test_positions_live PRIVATE "-framework CoreFoundation")
        find_library(PTHREAD_LIBRARY pthread)
        if(PTHREAD_LIBRARY)
            target_link_libraries(test_positions_live PRIVATE ${PTHREAD_LIBRARY})
        endif()
    elseif(UNIX)
        target_link_libraries(test_positions_live PRIVATE pthread)
    endif()

    message(STATUS "Position retrieval test program: test_positions_live")
endif()

# Simple connection diagnostic test
if(_build_native_cli AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/tests/test_simple_connect.cpp")
    add_executable(test_simple_connect ${CMAKE_CURRENT_SOURCE_DIR}/tests/test_simple_connect.cpp)

    target_include_directories(test_simple_connect
        PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${CMAKE_CURRENT_SOURCE_DIR}/src
            ${TWS_API_INCLUDE_DIR}
    )
    if(TWS_API_BUILD_VENDOR)
        target_include_directories(test_simple_connect PRIVATE "${TWS_API_EXTERNAL_BINARY_DIR}/proto_generated")
        add_dependencies(test_simple_connect twsapi_external)
    endif()

    target_link_libraries(test_simple_connect
        PRIVATE
            nlohmann_json::nlohmann_json
            spdlog::spdlog
            ${TWS_API_LIB}
            ${INTEL_DECIMAL_LIB}
    )

    target_sources(test_simple_connect PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client_impl_ewrapper.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/config_manager.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/rate_limiter.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/market_hours.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/pcap_capture.cpp
    )

    if(APPLE)
        target_link_libraries(test_simple_connect PRIVATE "-framework CoreFoundation")
        find_library(PTHREAD_LIBRARY pthread)
        if(PTHREAD_LIBRARY)
            target_link_libraries(test_simple_connect PRIVATE ${PTHREAD_LIBRARY})
        endif()
    elseif(UNIX)
        target_link_libraries(test_simple_connect PRIVATE pthread)
    endif()

    message(STATUS "Simple connection diagnostic: test_simple_connect")
endif()

# Deep diagnostic connection test with callback tracing
if(_build_native_cli AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/tests/test_diagnostic_connect.cpp")
    add_executable(test_diagnostic_connect ${CMAKE_CURRENT_SOURCE_DIR}/tests/test_diagnostic_connect.cpp)

    target_include_directories(test_diagnostic_connect
        PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${CMAKE_CURRENT_SOURCE_DIR}/src
            ${TWS_API_INCLUDE_DIR}
    )
    if(TWS_API_BUILD_VENDOR)
        target_include_directories(test_diagnostic_connect PRIVATE "${TWS_API_EXTERNAL_BINARY_DIR}/proto_generated")
        add_dependencies(test_diagnostic_connect twsapi_external)
    endif()

    target_link_libraries(test_diagnostic_connect
        PRIVATE
            nlohmann_json::nlohmann_json
            spdlog::spdlog
            ${TWS_API_LIB}
            ${INTEL_DECIMAL_LIB}
    )

    target_sources(test_diagnostic_connect PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client_impl_ewrapper.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/config_manager.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/rate_limiter.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/market_hours.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/pcap_capture.cpp
    )

    if(APPLE)
        target_link_libraries(test_diagnostic_connect PRIVATE "-framework CoreFoundation")
        find_library(PTHREAD_LIBRARY pthread)
        if(PTHREAD_LIBRARY)
            target_link_libraries(test_diagnostic_connect PRIVATE ${PTHREAD_LIBRARY})
        endif()
    elseif(UNIX)
        target_link_libraries(test_diagnostic_connect PRIVATE pthread)
    endif()

    message(STATUS "Deep diagnostic test: test_diagnostic_connect")
endif()

# Packet-level trace diagnostic
if(_build_native_cli AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/tests/test_packet_trace.cpp")
    add_executable(test_packet_trace ${CMAKE_CURRENT_SOURCE_DIR}/tests/test_packet_trace.cpp)

    target_include_directories(test_packet_trace
        PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${CMAKE_CURRENT_SOURCE_DIR}/src
            ${TWS_API_INCLUDE_DIR}
    )
    if(TWS_API_BUILD_VENDOR)
        target_include_directories(test_packet_trace PRIVATE "${TWS_API_EXTERNAL_BINARY_DIR}/proto_generated")
        add_dependencies(test_packet_trace twsapi_external)
    endif()

    target_link_libraries(test_packet_trace
        PRIVATE
            nlohmann_json::nlohmann_json
            spdlog::spdlog
            ${TWS_API_LIB}
            ${INTEL_DECIMAL_LIB}
    )

    target_sources(test_packet_trace PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client_impl_ewrapper.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/config_manager.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/rate_limiter.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/market_hours.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/pcap_capture.cpp
    )

    if(APPLE)
        target_link_libraries(test_packet_trace PRIVATE "-framework CoreFoundation")
        find_library(PTHREAD_LIBRARY pthread)
        if(PTHREAD_LIBRARY)
            target_link_libraries(test_packet_trace PRIVATE ${PTHREAD_LIBRARY})
        endif()
    elseif(UNIX)
        target_link_libraries(test_packet_trace PRIVATE pthread)
    endif()

    message(STATUS "Packet trace diagnostic: test_packet_trace")
endif()

# ====================================================================================
# Python Bindings (pybind11 or Cython)
# ============================================================================
if(ENABLE_PYTHON_BINDINGS)
    find_package(Python3 COMPONENTS Interpreter Development REQUIRED)

    if(PYTHON_BINDINGS_BACKEND STREQUAL "pybind11")
        set(PYBIND11_CPP_STANDARD ${CMAKE_CXX_STANDARD})
        fetchcontent_declare(
            pybind11
            GIT_REPOSITORY https://github.com/pybind/pybind11.git
            GIT_TAG v2.13.6
            GIT_SHALLOW TRUE
        )
        fetchcontent_makeavailable(pybind11)

        if(_build_native_cli AND TARGET pybind11::module)
            pybind11_add_module(box_spread_bindings
                src/box_spread_pybind.cpp
                src/types.cpp
                src/tws_conversions.cpp
                src/greeks_calculator.cpp
                src/strategies/box_spread/box_spread_strategy.cpp
                src/risk_calculator.cpp
                src/option_chain.cpp
                src/config_manager.cpp
                src/market_hours.cpp
            )
            target_include_directories(box_spread_bindings PRIVATE
                ${CMAKE_CURRENT_SOURCE_DIR}/include
                ${TWS_API_INCLUDE_DIR}
            )
            if(TWS_API_BUILD_VENDOR)
                target_include_directories(box_spread_bindings PRIVATE "${TWS_API_EXTERNAL_BINARY_DIR}/proto_generated")
                add_dependencies(box_spread_bindings twsapi_external)
            endif()
            target_link_libraries(box_spread_bindings PRIVATE
                nlohmann_json::nlohmann_json
                spdlog::spdlog
                Eigen3::Eigen
                QuantLib::QuantLib
                NLopt::nlopt
                ${Boost_LIBRARIES}
                ${TWS_API_LIB}
                ${INTEL_DECIMAL_LIB}
            )
            if(APPLE)
                target_link_libraries(box_spread_bindings PRIVATE "-framework CoreFoundation")
                find_library(PTHREAD_LIBRARY pthread)
                if(PTHREAD_LIBRARY)
                    target_link_libraries(box_spread_bindings PRIVATE ${PTHREAD_LIBRARY})
                endif()
            elseif(UNIX)
                target_link_libraries(box_spread_bindings PRIVATE pthread)
                find_library(UUID_LIBRARY uuid)
                if(UUID_LIBRARY)
                    target_link_libraries(box_spread_bindings PRIVATE ${UUID_LIBRARY})
                endif()
            endif()
            set_target_properties(box_spread_bindings PROPERTIES
                LIBRARY_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/python/bindings
                RUNTIME_OUTPUT_DIRECTORY ${CMAKE_SOURCE_DIR}/python/bindings
            )
            message(STATUS "Python bindings: pybind11 (target box_spread_bindings)")
            message(STATUS "  Build: cmake --build . --target box_spread_bindings")
        else()
            if(NOT _build_native_cli)
                message(STATUS "Python bindings (pybind11) skipped: native CLI not built")
            endif()
        endif()
    else()
        execute_process(
            COMMAND ${Python3_EXECUTABLE} -c "import Cython; print(Cython.__version__)"
            OUTPUT_VARIABLE CYTHON_VERSION
            OUTPUT_STRIP_TRAILING_WHITESPACE
            ERROR_QUIET
        )
        if(CYTHON_VERSION AND _build_native_cli)
            add_custom_target(
                python_bindings
                COMMAND ${Python3_EXECUTABLE} setup.py build_ext --inplace
                WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/python/bindings
                COMMENT "Building Python bindings with Cython"
            )
            message(STATUS "Python bindings: Cython (target python_bindings)")
            message(STATUS "  Build: cmake --build . --target python_bindings")
        elseif(NOT CYTHON_VERSION)
            message(WARNING "Cython not found. Install: pip install cython")
        endif()
    endif()
else()
    message(STATUS "Python bindings disabled.")
endif()

# ====================================================================================
# Testing
# ============================================================================
# Allow tests to be built even if native CLI is disabled, as tests may not require
# all the same dependencies (tests can use mock/stub implementations)
if(BUILD_TESTING)
    # Check if TWS API and Intel decimal libraries exist for tests
    # Tests can work with fallback paths defined in tests/CMakeLists.txt
    set(_can_build_tests ON)

    # Warn if dependencies are missing, but still try to build tests
    # (tests/CMakeLists.txt has fallback logic)
    if(NOT _native_prereqs_met)
        message(WARNING "Native CLI dependencies missing, but attempting to build tests anyway.")
        message(WARNING "Tests may fail if TWS API or Intel decimal libraries are not available.")
    endif()

    add_subdirectory(tests)
endif()

# ====================================================================================
# Documentation
# ============================================================================
if(BUILD_DOCS)
    find_package(Doxygen)
    if(DOXYGEN_FOUND)
        set(DOXYGEN_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/docs)
        doxygen_add_docs(docs
            ${CMAKE_CURRENT_SOURCE_DIR}/src
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            COMMENT "Generating documentation"
        )
    endif()
endif()

# ====================================================================================
# Summary
# ============================================================================
message(STATUS "============================================")
message(STATUS "  ${PROJECT_NAME} v${PROJECT_VERSION}")
message(STATUS "============================================")
message(STATUS "Build type:        ${CMAKE_BUILD_TYPE}")
message(STATUS "C++ Standard:      ${CMAKE_CXX_STANDARD}")
message(STATUS "Compiler:          ${CMAKE_CXX_COMPILER_ID}")
message(STATUS "Native CLI target: ${_build_native_cli}")
message(STATUS "Build testing:     ${BUILD_TESTING}")
message(STATUS "Enable LTO:        ${ENABLE_LTO}")
message(STATUS "Python bindings:   ${ENABLE_PYTHON_BINDINGS}")
message(STATUS "Install prefix:    ${CMAKE_INSTALL_PREFIX}")
if(NOT _native_prereqs_met)
    string(REPLACE ";" ", " _missing_native_deps_str "${_missing_native_deps}")
    message(STATUS "Missing native dependencies: ${_missing_native_deps_str}")
endif()
message(STATUS "============================================")
