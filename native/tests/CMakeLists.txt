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
    test_market_hours.cpp
    test_rate_limiter.cpp
    test_hedge_manager.cpp
    test_box_spread_bag.cpp
    eigen_integration_test.cpp
    test_convexity_calculator.cpp
    test_alpaca_adapter.cpp
    test_path_validator.cpp
    test_loan_position.cpp
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
    if(APPLE)
        set(_twsapi_ext ".dylib")
    else()
        set(_twsapi_ext ".so")
    endif()
    set(TWS_API_LIB
        "${TWS_API_INCLUDE_DIR}/build/lib/libtwsapi${_twsapi_ext}"
    )
endif()

if(NOT INTEL_DECIMAL_LIB)
    set(INTEL_DECIMAL_LIB
        "${CMAKE_SOURCE_DIR}/native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a"
    )
endif()

if(NOT ABSEIL_LIBS)
    if(APPLE)
        foreach(_dir /opt/homebrew/lib /usr/local/lib)
            file(GLOB _absl "${_dir}/libabsl*.dylib")
            if(_absl)
                set(ABSEIL_LIBS ${_absl})
                break()
            endif()
        endforeach()
        if(NOT ABSEIL_LIBS)
            file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.dylib")
        endif()
    else()
        file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.so" "/usr/lib/x86_64-linux-gnu/libabsl*.so")
    endif()
endif()

target_include_directories(box_spread_tests PRIVATE
    ${CMAKE_SOURCE_DIR}/native/include
    ${TWS_API_INCLUDE_DIR}
)

# Platform-specific protobuf for tests (TWS API dependency)
if(APPLE)
    find_library(Protobuf_LIBRARY NAMES protobuf libprotobuf
        PATHS /opt/homebrew/lib /usr/local/lib /usr/lib
        NO_DEFAULT_PATH
    )
    if(NOT Protobuf_LIBRARY)
        find_library(Protobuf_LIBRARY NAMES protobuf libprotobuf)
    endif()
    set(_test_protobuf_lib "${Protobuf_LIBRARY}")
    if(NOT _test_protobuf_lib)
        set(_test_protobuf_lib "/usr/local/lib/libprotobuf.dylib")
    endif()
elseif(UNIX)
    find_library(Protobuf_LIBRARY NAMES protobuf PATHS "/usr/lib/x86_64-linux-gnu" "/usr/lib64" "/usr/local/lib")
    if(Protobuf_LIBRARY)
        set(_test_protobuf_lib "${Protobuf_LIBRARY}")
    else()
        set(_test_protobuf_lib "protobuf")
    endif()
else()
    set(_test_protobuf_lib "protobuf")
endif()

target_link_libraries(box_spread_tests
    PRIVATE
        Catch2::Catch2WithMain
        nlohmann_json::nlohmann_json
        spdlog::spdlog
        CLI11::CLI11
        Eigen3::Eigen
        ${TWS_API_LIB}
        ${INTEL_DECIMAL_LIB}
        ${_test_protobuf_lib}
        ${ABSEIL_LIBS}
)

# Link Boost libraries (use same approach as main CMakeLists.txt)
# The parent CMakeLists.txt uses find_library and sets Boost_LIBRARIES
if(Boost_LIBRARIES)
    target_link_libraries(box_spread_tests PRIVATE ${Boost_LIBRARIES})
else()
    # Fallback: try to find Boost libraries directly in tests subdirectory
    # Check both Intel (/usr/local) and Apple Silicon (/opt/homebrew) Homebrew paths
    set(BOOST_CELLAR_PATHS "/opt/homebrew/Cellar/boost" "/usr/local/Cellar/boost")
    set(BOOST_CELLAR_DIR "")
    foreach(BOOST_CELLAR_PATH ${BOOST_CELLAR_PATHS})
        if(EXISTS "${BOOST_CELLAR_PATH}")
            file(GLOB BOOST_CELLAR_DIRS "${BOOST_CELLAR_PATH}/*")
            if(BOOST_CELLAR_DIRS)
                list(GET BOOST_CELLAR_DIRS 0 BOOST_CELLAR_DIR)
                break()
            endif()
        endif()
    endforeach()

    if(BOOST_CELLAR_DIR)
        set(BOOST_LIB_DIR "${BOOST_CELLAR_DIR}/lib")
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

        if(BOOST_DATE_TIME_LIB AND BOOST_FILESYSTEM_LIB)
            set(BOOST_TEST_LIBS "${BOOST_DATE_TIME_LIB}" "${BOOST_FILESYSTEM_LIB}")
            if(BOOST_SYSTEM_LIB)
                list(APPEND BOOST_TEST_LIBS "${BOOST_SYSTEM_LIB}")
            endif()
            target_link_libraries(box_spread_tests PRIVATE ${BOOST_TEST_LIBS})
        else()
            message(WARNING "Boost libraries not found - tests may fail")
        endif()
    else()
        message(WARNING "Boost not found - tests may fail")
    endif()
endif()

# Link NLopt if available (fetched via FetchContent)
if(TARGET NLopt::nlopt)
    target_link_libraries(box_spread_tests PRIVATE NLopt::nlopt)
else()
    message(WARNING "NLopt::nlopt target not found - tests may fail")
endif()

# Link QuantLib if available (fetched via FetchContent)
if(TARGET QuantLib::QuantLib)
    target_link_libraries(box_spread_tests PRIVATE QuantLib::QuantLib)
else()
    message(WARNING "QuantLib::QuantLib target not found - tests may fail")
endif()

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
# Note: CMAKE_SOURCE_DIR points to project root, so paths need native/ prefix
target_sources(box_spread_tests PRIVATE
    ${CMAKE_SOURCE_DIR}/native/src/config_manager.cpp
    ${CMAKE_SOURCE_DIR}/native/src/path_validator.cpp
    ${CMAKE_SOURCE_DIR}/native/src/tws_client.cpp
    ${CMAKE_SOURCE_DIR}/native/src/market_hours.cpp
    ${CMAKE_SOURCE_DIR}/native/src/rate_limiter.cpp
    ${CMAKE_SOURCE_DIR}/native/src/hedge_manager.cpp
    ${CMAKE_SOURCE_DIR}/native/src/strategies/box_spread/box_spread_bag.cpp
    ${CMAKE_SOURCE_DIR}/native/src/convexity_calculator.cpp
    ${CMAKE_SOURCE_DIR}/native/src/loan_position.cpp
    ${CMAKE_SOURCE_DIR}/native/src/loan_manager.cpp
)

# Conditionally include local source files only if NOT using extracted library
if(NOT USE_BOX_SPREAD_CPP_LIB)
    target_sources(box_spread_tests PRIVATE
        ${CMAKE_SOURCE_DIR}/native/src/option_chain.cpp
        ${CMAKE_SOURCE_DIR}/native/src/strategies/box_spread/box_spread_strategy.cpp
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
