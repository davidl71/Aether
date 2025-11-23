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
option(ENABLE_PYTHON_BINDINGS "Enable Python bindings (Cython)" ON)
option(ENABLE_NAUTILUS_TRADER "Enable nautilus_trader integration" ON)
option(ENABLE_DISTCC "Enable distcc distributed compilation" OFF)
option(ENABLE_CCACHE "Enable ccache compilation caching" OFF)
option(ENABLE_SCCACHE "Enable sccache compilation caching" OFF)
option(ENABLE_NATIVE_CLI "Build the native ib_box_spread CLI" ON)

# ============================================================================
# C++ Standard and Compiler Requirements
# ============================================================================
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Export compile commands for IDE support
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

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
)

set(SPDLOG_REPOSITORY "https://github.com/gabime/spdlog.git")
fetchcontent_declare(
    spdlog
    GIT_REPOSITORY ${SPDLOG_REPOSITORY}
    GIT_TAG v1.13.0
    GIT_SHALLOW TRUE
)

set(CLI11_REPOSITORY "https://github.com/CLIUtils/CLI11.git")
fetchcontent_declare(
    CLI11
    GIT_REPOSITORY ${CLI11_REPOSITORY}
    GIT_TAG v2.4.1
    GIT_SHALLOW TRUE
)

set(FTXUI_REPOSITORY "https://github.com/ArthurSonzogni/FTXUI.git")
fetchcontent_declare(
    ftxui
    GIT_REPOSITORY ${FTXUI_REPOSITORY}
    GIT_TAG v5.0.0
    GIT_SHALLOW TRUE
)

set(EIGEN3_REPOSITORY "https://gitlab.com/libeigen/eigen.git")
fetchcontent_declare(
    Eigen3
    GIT_REPOSITORY ${EIGEN3_REPOSITORY}
    GIT_TAG 3.4.0
    GIT_SHALLOW TRUE
)

# Find Boost (required for QuantLib)
# Set Boost root if installed via Homebrew
set(BOOST_FOUND_DIRECTLY FALSE)
if(APPLE AND EXISTS "/usr/local/Cellar/boost")
    # Find the actual Boost version directory
    file(GLOB BOOST_CELLAR_DIRS "/usr/local/Cellar/boost/*")
    if(BOOST_CELLAR_DIRS)
        list(GET BOOST_CELLAR_DIRS 0 BOOST_CELLAR_DIR)
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
endif()

# Try find_package if direct finding didn't work
if(NOT BOOST_FOUND_DIRECTLY)
    set(Boost_NO_BOOST_CMAKE ON)
    find_package(Boost QUIET COMPONENTS date_time filesystem system)
    if(NOT Boost_FOUND)
        message(FATAL_ERROR "Could not find Boost. Please install via: brew install boost")
    endif()
endif()

# QuantLib via FetchContent
set(QUANTLIB_REPOSITORY "https://github.com/lballabio/quantlib.git")
fetchcontent_declare(
    QuantLib
    GIT_REPOSITORY ${QUANTLIB_REPOSITORY}
    GIT_TAG QuantLib-v1.32
    GIT_SHALLOW TRUE
)

# NLopt via FetchContent
set(NLOPT_REPOSITORY "https://github.com/stevengj/nlopt.git")
fetchcontent_declare(
    NLopt
    GIT_REPOSITORY ${NLOPT_REPOSITORY}
    GIT_TAG v2.7.1
    GIT_SHALLOW TRUE
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

fetchcontent_makeavailable(nlohmann_json spdlog CLI11 ftxui Eigen3 QuantLib NLopt)

if(BUILD_TESTING)
    set(CATCH2_REPOSITORY "https://github.com/catchorg/Catch2.git")
    fetchcontent_declare(
        Catch2
        GIT_REPOSITORY ${CATCH2_REPOSITORY}
        GIT_TAG v3.5.2
        GIT_SHALLOW TRUE
    )
    fetchcontent_makeavailable(Catch2)
    include(CTest)
    enable_testing()
endif()

# ====================================================================================
# TWS API (IBKR C++ API)
# ============================================================================
set(TWS_API_VENDOR_ROOT "${CMAKE_CURRENT_SOURCE_DIR}")
cmake_path(APPEND TWS_API_VENDOR_ROOT
    "third_party" "tws-api" "IBJts" "source" "cppclient"
)

set(DEFAULT_TWS_API_INCLUDE_DIR "${TWS_API_VENDOR_ROOT}")
cmake_path(APPEND DEFAULT_TWS_API_INCLUDE_DIR "client")
set(
    TWS_API_INCLUDE_DIR
    "${DEFAULT_TWS_API_INCLUDE_DIR}"
    CACHE PATH "Path to TWS API include directory"
)

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

if(EXISTS "${TWS_API_INCLUDE_DIR}")
    message(STATUS "Using TWS API include dir: ${TWS_API_INCLUDE_DIR}")
else()
    message(WARNING "TWS API headers not found at ${TWS_API_INCLUDE_DIR}")
    message(WARNING "Download from: https://interactivebrokers.github.io/")
    message(WARNING "Extract to: ${CMAKE_CURRENT_SOURCE_DIR}/third_party/tws-api/")
endif()

# Determine library extension based on platform
if(APPLE)
    set(_twsapi_lib_ext ".dylib")
elseif(UNIX)
    set(_twsapi_lib_ext ".so")
else()
    set(_twsapi_lib_ext ".dll")
endif()

set(DEFAULT_TWS_API_LIB "${TWS_API_VENDOR_ROOT}")
cmake_path(APPEND DEFAULT_TWS_API_LIB "client" "build" "lib" "libtwsapi${_twsapi_lib_ext}")
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

if(TWS_API_BUILD_VENDOR)
    if(NOT EXISTS "${TWS_API_VENDOR_ROOT}/../../CMakeLists.txt")
        message(FATAL_ERROR "Vendored TWS API source not found at ${TWS_API_VENDOR_ROOT}/../../CMakeLists.txt")
    endif()
    include(ExternalProject)
    set(TWS_API_EXTERNAL_BINARY_DIR "${CMAKE_BINARY_DIR}/tws_api_vendor_build")
    ExternalProject_Add(twsapi_external
        SOURCE_DIR "${TWS_API_VENDOR_ROOT}/../.."
        BINARY_DIR "${TWS_API_EXTERNAL_BINARY_DIR}"
        CMAKE_ARGS
            -DCMAKE_BUILD_TYPE=${CMAKE_BUILD_TYPE}
            -DCMAKE_INSTALL_PREFIX=${TWS_API_EXTERNAL_BINARY_DIR}/install
            -DCMAKE_POLICY_DEFAULT_CMP0126=NEW
    )

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

set(INTEL_DECIMAL_LIB "${CMAKE_CURRENT_SOURCE_DIR}")
cmake_path(APPEND INTEL_DECIMAL_LIB "third_party" "IntelRDFPMathLib20U2" "LIBRARY" "libbid.a")

# Find Abseil libraries based on platform
if(APPLE)
    file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.dylib")
elseif(UNIX)
    file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.so")
    if(NOT ABSEIL_LIBS)
        file(GLOB ABSEIL_LIBS "/usr/lib/x86_64-linux-gnu/libabsl*.so")
    endif()
    if(NOT ABSEIL_LIBS)
        file(GLOB ABSEIL_LIBS "/usr/lib/libabsl*.so")
    endif()
endif()

# Find protobuf library based on platform
if(APPLE)
    set(PROTOBUF_LIB "/usr/local/lib/libprotobuf.dylib")
elseif(UNIX)
    # Try common Linux locations for protobuf
    find_library(PROTOBUF_LIB
        NAMES protobuf
        PATHS
            /usr/local/lib
            /usr/lib/x86_64-linux-gnu
            /usr/lib
        NO_DEFAULT_PATH
    )
    if(NOT PROTOBUF_LIB)
        # Fallback to explicit path
        set(PROTOBUF_LIB "/usr/local/lib/libprotobuf.so")
    endif()
endif()

set(_missing_native_deps "")
set(_native_prereqs_met TRUE)

if(NOT TWS_API_LIB OR NOT EXISTS "${TWS_API_LIB}")
    list(APPEND _missing_native_deps "TWS API shared library")
    set(_native_prereqs_met FALSE)
endif()

if(NOT EXISTS "${INTEL_DECIMAL_LIB}")
    list(APPEND _missing_native_deps "Intel decimal math library (libbid.a)")
    set(_native_prereqs_met FALSE)
endif()

if(NOT ABSEIL_LIBS)
    message(STATUS "Abseil libraries not detected in /usr/local/lib; continuing without optional Abseil linkage.")
endif()

if(ENABLE_NATIVE_CLI AND NOT _native_prereqs_met)
    string(REPLACE ";" ", " _missing_native_deps_str "${_missing_native_deps}")
    message(WARNING "Native CLI prerequisites missing: ${_missing_native_deps_str}. The native binary will be skipped.")
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
        src/rate_limiter.cpp
        src/config_manager.cpp
        src/nats_client.cpp
        src/pcap_capture.cpp
        src/convexity_calculator.cpp
    )

    set(HEADERS
        include/tws_client.h
        include/rate_limiter.h
        include/config_manager.h
        include/pcap_capture.h
        include/convexity_calculator.h
        include/types.h
    )

    # Conditionally include local source files only if NOT using extracted library
    if(NOT USE_BOX_SPREAD_CPP_LIB)
        list(APPEND SOURCES
            src/option_chain.cpp
            src/strategies/box_spread/box_spread_strategy.cpp
            src/strategies/box_spread/box_spread_bag.cpp
            src/order_manager.cpp
            src/risk_calculator.cpp
        )
        list(APPEND HEADERS
            include/option_chain.h
            include/strategies/box_spread/box_spread_strategy.h
            include/strategies/box_spread/box_spread_bag.h
            include/order_manager.h
            include/risk_calculator.h
        )
        message(STATUS "Including local box spread implementation files")
    else()
        # When using extracted library, include TWS adapter
        list(APPEND SOURCES
            src/brokers/tws_adapter.cpp
        )
        list(APPEND HEADERS
            include/brokers/tws_adapter.h
        )
        message(STATUS "Excluding local box spread files (using extracted library with TWS adapter)")
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

    # Include the generated version header in the build
    target_include_directories(ib_box_spread
        PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${CMAKE_CURRENT_BINARY_DIR}/include  # For generated version.h
            ${TWS_API_INCLUDE_DIR}
    )

    # Add box-spread-cpp library include directory if using extracted library
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
            ${PROTOBUF_LIB}
            ${ABSEIL_LIBS}
    )

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
# TUI Executable (C++)
# ============================================================================
option(ENABLE_TUI "Build the C++ TUI application" ON)
option(ENABLE_TA_LIB "Enable TA-Lib technical analysis library integration" OFF)

if(ENABLE_TUI)
    set(TUI_SOURCES
        src/tui_app.cpp
        src/tui_provider.cpp
        src/tui_breadcrumb.cpp
        src/tui_converter.cpp
        src/tui_config.cpp
        src/tui_data.cpp
        src/types_utils.cpp
    )

    set(TUI_HEADERS
        include/tui_data.h
        include/tui_provider.h
        include/tui_converter.h
        include/tui_config.h
        include/tui_breadcrumb.h
    )

    add_executable(ib_box_spread_tui ${TUI_SOURCES} ${TUI_HEADERS})

    target_include_directories(ib_box_spread_tui
        PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${CMAKE_CURRENT_BINARY_DIR}/include
    )

    # FTXUI is header-only, but we need to link against its dependencies
    # Find libcurl for HTTP client support in TUI
    find_package(CURL REQUIRED)

    target_link_libraries(ib_box_spread_tui
        PRIVATE
            nlohmann_json::nlohmann_json
            spdlog::spdlog
            CURL::libcurl
    )

    # FTXUI uses FetchContent, link to its interface target if available
    # FTXUI v5.0.0 uses component targets that need to be linked
    if(TARGET ftxui::ftxui)
        target_link_libraries(ib_box_spread_tui PRIVATE ftxui::ftxui)
    elseif(TARGET ftxui::component)
        # Link component library if available
        target_link_libraries(ib_box_spread_tui PRIVATE ftxui::component)
    else()
        # Fallback: add include directory manually and link any built libraries
        get_filename_component(FTXUI_INCLUDE_DIR "${CMAKE_BINARY_DIR}/_deps/ftxui-src/include" ABSOLUTE)
        if(EXISTS "${FTXUI_INCLUDE_DIR}")
            target_include_directories(ib_box_spread_tui PRIVATE "${FTXUI_INCLUDE_DIR}")
        endif()
        # Try to find and link FTXUI component library
        find_library(FTXUI_COMPONENT_LIB
            NAMES ftxui-component
            PATHS "${CMAKE_BINARY_DIR}/_deps/ftxui-build"
            NO_DEFAULT_PATH
        )
        if(FTXUI_COMPONENT_LIB)
            target_link_libraries(ib_box_spread_tui PRIVATE ${FTXUI_COMPONENT_LIB})
        endif()
    endif()

    if(APPLE)
        target_link_libraries(ib_box_spread_tui PRIVATE "-framework CoreFoundation")
        find_library(PTHREAD_LIBRARY pthread)
        if(PTHREAD_LIBRARY)
            target_link_libraries(ib_box_spread_tui PRIVATE ${PTHREAD_LIBRARY})
        endif()
    elseif(UNIX)
        target_link_libraries(ib_box_spread_tui PRIVATE pthread)
    endif()

    install(TARGETS ib_box_spread_tui
        RUNTIME DESTINATION bin
    )

    message(STATUS "TUI executable: ib_box_spread_tui")

    # TA-Lib integration (optional)
    # TA-Lib is a C/C++ library for technical analysis indicators
    # See: https://ta-lib.org/
    # Installation:
    #   macOS: brew install ta-lib
    #   Linux: Download from https://sourceforge.net/projects/ta-lib/
    #   Windows: Download pre-built binaries
    if(ENABLE_TA_LIB)
        find_library(TA_LIB_LIBRARY
            NAMES ta_lib ta_libc
            PATHS
                /usr/local/lib
                /opt/homebrew/lib
                /usr/lib
        )
        find_path(TA_LIB_INCLUDE_DIR
            NAMES ta_libc.h
            PATHS
                /usr/local/include
                /opt/homebrew/include
                /usr/include
        )

        if(TA_LIB_LIBRARY AND TA_LIB_INCLUDE_DIR)
            target_include_directories(ib_box_spread_tui PRIVATE ${TA_LIB_INCLUDE_DIR})
            target_link_libraries(ib_box_spread_tui PRIVATE ${TA_LIB_LIBRARY})
            target_compile_definitions(ib_box_spread_tui PRIVATE ENABLE_TA_LIB)
            message(STATUS "TA-Lib found: ${TA_LIB_LIBRARY}")
            message(STATUS "TA-Lib include: ${TA_LIB_INCLUDE_DIR}")
        else()
            message(WARNING "TA-Lib not found. Install with: brew install ta-lib (macOS) or download from https://ta-lib.org/")
            message(WARNING "Continuing without TA-Lib support")
        endif()
    endif()
endif()

# ============================================================================
# TWS Connection Test Program
# ============================================================================
if(_build_native_cli AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/../scripts/test_tws_connection.cpp")
    add_executable(test_tws_connection ${CMAKE_CURRENT_SOURCE_DIR}/../scripts/test_tws_connection.cpp)

    target_include_directories(test_tws_connection
        PRIVATE
            ${CMAKE_CURRENT_SOURCE_DIR}/include
            ${CMAKE_CURRENT_SOURCE_DIR}/src
            ${TWS_API_INCLUDE_DIR}
    )

    target_link_libraries(test_tws_connection
        PRIVATE
            nlohmann_json::nlohmann_json
            spdlog::spdlog
            ${TWS_API_LIB}
            ${INTEL_DECIMAL_LIB}
            ${PROTOBUF_LIB}
            ${ABSEIL_LIBS}
    )

    # Link with TWS client source files
    target_sources(test_tws_connection PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/src/tws_client.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/config_manager.cpp
        ${CMAKE_CURRENT_SOURCE_DIR}/src/rate_limiter.cpp
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

# ====================================================================================
# Python Bindings (Cython)
# ============================================================================
if(ENABLE_PYTHON_BINDINGS AND _build_native_cli)
    find_package(Python3 COMPONENTS Interpreter Development REQUIRED)

    execute_process(
        COMMAND ${Python3_EXECUTABLE} -c "import Cython; print(Cython.__version__)"
        OUTPUT_VARIABLE CYTHON_VERSION
        OUTPUT_STRIP_TRAILING_WHITESPACE
        ERROR_QUIET
    )

    if(CYTHON_VERSION)
        message(STATUS "Found Cython: ${CYTHON_VERSION}")
        add_custom_target(
            python_bindings
            COMMAND ${Python3_EXECUTABLE} setup.py build_ext --inplace
            WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/python/bindings
            COMMENT "Building Python bindings with Cython"
        )
        message(STATUS "Python bindings target: python_bindings")
        message(STATUS "  Build with: cmake --build . --target python_bindings")
        message(STATUS "  Or install with: cd python/bindings && pip install -e .")
    else()
        message(WARNING "Cython not found. Install with: pip install cython. Python bindings target skipped.")
    endif()
elseif(ENABLE_PYTHON_BINDINGS)
    message(STATUS "Python bindings disabled because the native CLI is not being built.")
endif()

# ====================================================================================
# Testing
# ============================================================================
if(BUILD_TESTING AND _build_native_cli)
    add_subdirectory(tests)
elseif(BUILD_TESTING AND NOT _build_native_cli)
    message(STATUS "Skipping native C++ tests because the native CLI target is not being built.")
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
message(STATUS "NautilusTrader:    ${ENABLE_NAUTILUS_TRADER}")
message(STATUS "Install prefix:    ${CMAKE_INSTALL_PREFIX}")
if(NOT _native_prereqs_met)
    string(REPLACE ";" ", " _missing_native_deps_str "${_missing_native_deps}")
    message(STATUS "Missing native dependencies: ${_missing_native_deps_str}")
endif()
message(STATUS "============================================")
