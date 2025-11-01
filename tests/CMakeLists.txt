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
)

add_executable(box_spread_tests ${TEST_SOURCES})

# ============================================================================
# Include Directories
# ============================================================================

target_include_directories(box_spread_tests
    PRIVATE
        ${CMAKE_SOURCE_DIR}/include
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
)

# Link with the same object files as the main executable
# (This avoids code duplication)
target_sources(box_spread_tests PRIVATE
    ${CMAKE_SOURCE_DIR}/src/config_manager.cpp
    ${CMAKE_SOURCE_DIR}/src/tws_client.cpp
    ${CMAKE_SOURCE_DIR}/src/option_chain.cpp
    ${CMAKE_SOURCE_DIR}/src/box_spread_strategy.cpp
    ${CMAKE_SOURCE_DIR}/src/order_manager.cpp
    ${CMAKE_SOURCE_DIR}/src/risk_calculator.cpp
)

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
