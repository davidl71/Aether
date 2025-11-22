# tests/CMakeLists.txt
cmake_minimum_required(VERSION 3.21)

# ============================================================================
# Test Executable
# ============================================================================

set(TEST_SOURCES
    test_main.cpp
    test_config_manager.cpp
    test_box_spread_strategy.cpp
    test_risk_calculator.cpp
    test_order_manager.cpp
    test_tws_client.cpp
    test_tws_integration.cpp
    test_market_data_integration.cpp
    test_box_spread_e2e.cpp
    test_option_chain.cpp
    test_rate_limiter.cpp
    test_hedge_manager.cpp
    test_box_spread_bag.cpp
    eigen_integration_test.cpp
    test_convexity_calculator.cpp
)

add_executable(box_spread_tests ${TEST_SOURCES})

# ============================================================================
# Include Directories
# ============================================================================

# TWS API configuration (needed for tws_client.cpp). These variables are also
# defined in the parent build; fall back to the vendored paths if they were not
# provided via cache entries.
if(NOT TWS_API_INCLUDE_DIR)
    set(TWS_API_INCLUDE_DIR
        "${CMAKE_SOURCE_DIR}/native/third_party/tws-api/IBJts/source/cppclient/client"
    )
endif()

if(NOT TWS_API_LIB)
    set(TWS_API_LIB
        "${TWS_API_INCLUDE_DIR}/build/lib/libtwsapi.dylib"
    )
endif()

if(NOT INTEL_DECIMAL_LIB)
    set(INTEL_DECIMAL_LIB
        "${CMAKE_SOURCE_DIR}/native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a"
    )
endif()

if(NOT ABSEIL_LIBS)
    file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.dylib")
endif()

target_include_directories(box_spread_tests PRIVATE
    ${CMAKE_SOURCE_DIR}/native/include
    ${TWS_API_INCLUDE_DIR}
)

# ============================================================================
# Link Libraries
# ============================================================================

target_link_libraries(box_spread_tests
    PRIVATE
        Catch2::Catch2WithMain
        nlohmann_json::nlohmann_json
        spdlog::spdlog
        CLI11::CLI11
        Eigen3::Eigen
        QuantLib::QuantLib
        Boost::date_time
        Boost::filesystem
        Boost::system
        NLopt::nlopt
        ${TWS_API_LIB}
        ${INTEL_DECIMAL_LIB}
        /usr/local/lib/libprotobuf.dylib
        ${ABSEIL_LIBS}
)

# Platform-specific libraries (needed for TWS API)
if(APPLE)
    target_link_libraries(box_spread_tests PRIVATE "-framework CoreFoundation")
    find_library(PTHREAD_LIBRARY pthread)
    if(PTHREAD_LIBRARY)
        target_link_libraries(box_spread_tests PRIVATE ${PTHREAD_LIBRARY})
    endif()
elseif(UNIX)
    target_link_libraries(box_spread_tests PRIVATE pthread)
endif()

# Link with the same object files as the main executable
# (This avoids code duplication)
target_sources(box_spread_tests PRIVATE
    ${CMAKE_SOURCE_DIR}/native/src/config_manager.cpp
    ${CMAKE_SOURCE_DIR}/native/src/tws_client.cpp
    ${CMAKE_SOURCE_DIR}/native/src/rate_limiter.cpp
    ${CMAKE_SOURCE_DIR}/native/src/hedge_manager.cpp
    ${CMAKE_SOURCE_DIR}/native/src/box_spread_bag.cpp
    ${CMAKE_SOURCE_DIR}/native/src/convexity_calculator.cpp
)

# Conditionally include local source files only if NOT using extracted library
if(NOT USE_BOX_SPREAD_CPP_LIB)
    target_sources(box_spread_tests PRIVATE
        ${CMAKE_SOURCE_DIR}/native/src/option_chain.cpp
        ${CMAKE_SOURCE_DIR}/native/src/box_spread_strategy.cpp
        ${CMAKE_SOURCE_DIR}/native/src/order_manager.cpp
        ${CMAKE_SOURCE_DIR}/native/src/risk_calculator.cpp
    )
endif()

# Link to extracted box-spread-cpp library if enabled
if(USE_BOX_SPREAD_CPP_LIB AND TARGET box_spread_cpp)
    target_link_libraries(box_spread_tests PRIVATE box_spread_cpp)
    message(STATUS "Linked box_spread_tests to box_spread_cpp library")
endif()

# ============================================================================
# Test Discovery
# ============================================================================

include(Catch)
catch_discover_tests(box_spread_tests)

# ============================================================================
# Test Target
# ============================================================================

add_custom_target(run_tests
    COMMAND box_spread_tests
    DEPENDS box_spread_tests
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Running tests..."
)
