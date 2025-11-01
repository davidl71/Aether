cmake_minimum_required(VERSION 3.21)

# ============================================================================
# Project Configuration
# ============================================================================
project(IBKRBoxSpreadGenerator
    VERSION 1.0.0
    DESCRIPTION "Interactive Brokers Box Spread Trading Application"
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

# ============================================================================
# C++ Standard and Compiler Requirements
# ============================================================================
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Export compile commands for IDE support
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

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
# Dependencies
# ============================================================================
include(FetchContent)

# JSON library for configuration
FetchContent_Declare(
    nlohmann_json
    URL https://github.com/nlohmann/json/releases/download/v3.11.3/json.tar.xz
    URL_HASH SHA256=d6c65aca6b1ed68e7a182f4757257b107ae403032760ed6ef121c9d55e81757d
)

# Logging library
FetchContent_Declare(
    spdlog
    GIT_REPOSITORY https://github.com/gabime/spdlog.git
    GIT_TAG v1.13.0
    GIT_SHALLOW TRUE
)

# Command-line parsing
FetchContent_Declare(
    CLI11
    GIT_REPOSITORY https://github.com/CLIUtils/CLI11.git
    GIT_TAG v2.4.1
    GIT_SHALLOW TRUE
)

FetchContent_MakeAvailable(nlohmann_json spdlog CLI11)

# Testing framework
if(BUILD_TESTING)
    FetchContent_Declare(
        Catch2
        GIT_REPOSITORY https://github.com/catchorg/Catch2.git
        GIT_TAG v3.5.2
        GIT_SHALLOW TRUE
    )
    FetchContent_MakeAvailable(Catch2)
    include(CTest)
    enable_testing()
endif()

# TWS API (IBKR C++ API) - Must be manually downloaded
# Download from: https://interactivebrokers.github.io/
set(TWS_API_INCLUDE_DIR "${CMAKE_SOURCE_DIR}/vendor/tws-api/IBJts/source/cppclient/client"
    CACHE PATH "Path to TWS API include directory")

if(EXISTS "${TWS_API_INCLUDE_DIR}")
    message(STATUS "TWS API found: ${TWS_API_INCLUDE_DIR}")
else()
    message(WARNING "TWS API not found at ${TWS_API_INCLUDE_DIR}")
    message(WARNING "Download from: https://interactivebrokers.github.io/")
    message(WARNING "Extract to: ${CMAKE_SOURCE_DIR}/vendor/tws-api/")
endif()

# ============================================================================
# Source Files
# ============================================================================
set(SOURCES
    src/ib_box_spread.cpp
    src/tws_client.cpp
    src/option_chain.cpp
    src/box_spread_strategy.cpp
    src/order_manager.cpp
    src/risk_calculator.cpp
    src/config_manager.cpp
)

set(HEADERS
    include/tws_client.h
    include/option_chain.h
    include/box_spread_strategy.h
    include/order_manager.h
    include/risk_calculator.h
    include/config_manager.h
    include/types.h
)

# TWS API Library (pre-built with all dependencies)
set(TWS_API_LIB "${CMAKE_SOURCE_DIR}/vendor/tws-api/IBJts/source/cppclient/client/build/lib/libtwsapi.dylib")
set(INTEL_DECIMAL_LIB "${CMAKE_SOURCE_DIR}/vendor/IntelRDFPMathLib20U2/LIBRARY/libbid.a")
file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.dylib")

# ============================================================================
# Main Executable
# ============================================================================
add_executable(ib_box_spread ${SOURCES} ${HEADERS})

target_include_directories(ib_box_spread
    PRIVATE
        ${CMAKE_CURRENT_SOURCE_DIR}/include
        ${TWS_API_INCLUDE_DIR}
)

target_link_libraries(ib_box_spread
    PRIVATE
        nlohmann_json::nlohmann_json
        spdlog::spdlog
        CLI11::CLI11
        ${TWS_API_LIB}
        ${INTEL_DECIMAL_LIB}
        /usr/local/lib/libprotobuf.dylib
        ${ABSEIL_LIBS}
)

# Platform-specific libraries
if(APPLE)
    target_link_libraries(ib_box_spread PRIVATE "-framework CoreFoundation")
    find_library(PTHREAD_LIBRARY pthread)
    if(PTHREAD_LIBRARY)
        target_link_libraries(ib_box_spread PRIVATE ${PTHREAD_LIBRARY})
    endif()
elseif(UNIX)
    target_link_libraries(ib_box_spread PRIVATE pthread)
endif()

# ============================================================================
# Installation
# ============================================================================
install(TARGETS ib_box_spread
    RUNTIME DESTINATION bin
)

install(FILES README.md
    DESTINATION share/doc/ib_box_spread
)

install(DIRECTORY config/
    DESTINATION share/ib_box_spread/config
    FILES_MATCHING PATTERN "*.example.json"
)

# ============================================================================
# Testing
# ============================================================================
if(BUILD_TESTING)
    add_subdirectory(tests)
endif()

# ============================================================================
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

# ============================================================================
# Summary
# ============================================================================
message(STATUS "============================================")
message(STATUS "  ${PROJECT_NAME} v${PROJECT_VERSION}")
message(STATUS "============================================")
message(STATUS "Build type:    ${CMAKE_BUILD_TYPE}")
message(STATUS "C++ Standard:  ${CMAKE_CXX_STANDARD}")
message(STATUS "Compiler:      ${CMAKE_CXX_COMPILER_ID}")
message(STATUS "Build testing: ${BUILD_TESTING}")
message(STATUS "Enable LTO:    ${ENABLE_LTO}")
message(STATUS "Install prefix: ${CMAKE_INSTALL_PREFIX}")
message(STATUS "============================================")
