# tests/CMakeLists.txt
cmake_minimum_required(VERSION 3.21)

# ============================================================================
# Test Executable
# ============================================================================

set(CORE_TEST_SOURCES
    test_main.cpp
    test_config_manager.cpp
    test_market_hours.cpp
    test_rate_limiter.cpp
    test_convexity_calculator.cpp
    # test_path_validator.cpp  # DEPRECATED: path_validator removed
    # test_asset_relationship.cpp  # DEPRECATED: financing_instrument removed
    # test_loan_position.cpp  # TODO: File missing
    test_amortization_schedule.cpp
    test_yield_curve_fitting.cpp
)

set(BOX_SPREAD_TEST_SOURCES
    test_main.cpp
    test_box_spread_strategy.cpp
    test_risk_calculator.cpp
    test_box_spread_e2e.cpp
    test_option_chain.cpp
    test_hedge_manager.cpp
    test_box_spread_bag.cpp
    eigen_integration_test.cpp
    test_greeks_calculator.cpp
)

set(TWS_TEST_SOURCES
    test_main.cpp
    test_order_manager.cpp
    test_tws_client.cpp
    test_tws_integration.cpp
    test_market_data_integration.cpp
)

add_executable(box_spread_core_tests ${CORE_TEST_SOURCES})
add_executable(box_spread_tests ${BOX_SPREAD_TEST_SOURCES})
add_executable(box_spread_tws_tests ${TWS_TEST_SOURCES})

# ============================================================================
# Include Directories
# ============================================================================

# TWS API configuration (needed for tws_client.cpp). These variables are also
# defined in the parent build; fall back to the vendored paths if they were not
# provided via cache entries.
if(NOT TWS_API_INCLUDE_DIR)
    if(EXISTS "${CMAKE_SOURCE_DIR}/../tws-api/source/cppclient/client")
        set(TWS_API_INCLUDE_DIR
            "${CMAKE_SOURCE_DIR}/../tws-api/source/cppclient/client"
        )
    else()
        set(TWS_API_INCLUDE_DIR
            "${CMAKE_SOURCE_DIR}/native/third_party/tws-api/source/cppclient/client"
        )
    endif()
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
    # DEPRECATED: IntelRDFPMathLib20U2 support will be removed. Prefer U4.
    set(_test_u4 "${CMAKE_SOURCE_DIR}/native/third_party/IntelRDFPMathLib20U4/LIBRARY/libbid.a")
    set(_test_u2 "${CMAKE_SOURCE_DIR}/native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a")
    if(EXISTS "${_test_u4}")
        set(INTEL_DECIMAL_LIB "${_test_u4}")
    elseif(EXISTS "${_test_u2}")
        message(WARNING "IntelRDFPMathLib20U2 is deprecated. Please run ./scripts/fetch_third_party.sh to get U4.")
        set(INTEL_DECIMAL_LIB "${_test_u2}")
    else()
        message(FATAL_ERROR "Intel decimal math library not found. Run ./scripts/fetch_third_party.sh")
    endif()
    unset(_test_u4)
    unset(_test_u2)
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

function(configure_native_test_target target_name)
    target_include_directories(${target_name} PRIVATE
        ${CMAKE_SOURCE_DIR}/native/include
    )
    target_link_libraries(${target_name}
        PRIVATE
            Catch2::Catch2WithMain
            nlohmann_json::nlohmann_json
            spdlog::spdlog
            Eigen3::Eigen
    )
endfunction()

configure_native_test_target(box_spread_core_tests)
configure_native_test_target(box_spread_tests)
configure_native_test_target(box_spread_tws_tests)

# When TWS_API_BUILD_VENDOR: prepend proto_generated so system-protobuf .pb.h
# files shadow the vendored ones (same pattern as ib_box_spread target).
if(TWS_API_BUILD_VENDOR AND DEFINED TWS_API_EXTERNAL_BINARY_DIR)
    target_include_directories(box_spread_tests PRIVATE "${TWS_API_EXTERNAL_BINARY_DIR}/proto_generated")
    target_include_directories(box_spread_tws_tests PRIVATE "${TWS_API_EXTERNAL_BINARY_DIR}/proto_generated")
endif()
target_include_directories(box_spread_tests PRIVATE ${TWS_API_INCLUDE_DIR})
target_include_directories(box_spread_tws_tests PRIVATE ${TWS_API_INCLUDE_DIR})

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

target_link_libraries(box_spread_core_tests
    PRIVATE
        NLopt::nlopt
)

target_link_libraries(box_spread_tests
    PRIVATE
        CLI11::CLI11
        Eigen3::Eigen
        ${TWS_API_LIB}
        ${INTEL_DECIMAL_LIB}
        ${_test_protobuf_lib}
        ${ABSEIL_LIBS}
)

target_link_libraries(box_spread_tws_tests
    PRIVATE
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
    target_link_libraries(box_spread_tws_tests PRIVATE ${Boost_LIBRARIES})
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
            target_link_libraries(box_spread_tws_tests PRIVATE ${BOOST_TEST_LIBS})
        else()
            message(WARNING "Boost libraries not found - tests may fail")
        endif()
    else()
        message(WARNING "Boost not found - tests may fail")
    endif()
endif()

if(NOT TARGET NLopt::nlopt)
    message(WARNING "NLopt::nlopt target not found - core convexity tests may fail")
endif()

if(TARGET QuantLib::QuantLib)
    target_link_libraries(box_spread_tests PRIVATE QuantLib::QuantLib)
else()
    message(WARNING "QuantLib::QuantLib target not found - option/greeks tests may fail")
endif()

# Platform-specific libraries (needed for TWS API)
if(APPLE)
    target_link_libraries(box_spread_tests PRIVATE "-framework CoreFoundation")
    target_link_libraries(box_spread_tws_tests PRIVATE "-framework CoreFoundation")
    find_library(PTHREAD_LIBRARY pthread)
    if(PTHREAD_LIBRARY)
        target_link_libraries(box_spread_tests PRIVATE ${PTHREAD_LIBRARY})
        target_link_libraries(box_spread_tws_tests PRIVATE ${PTHREAD_LIBRARY})
    endif()
elseif(UNIX)
    target_link_libraries(box_spread_tests PRIVATE pthread)
    target_link_libraries(box_spread_tws_tests PRIVATE pthread)
endif()

# libuuid needed by nats_client.cpp
find_library(UUID_LIBRARY uuid)
if(UUID_LIBRARY)
    target_link_libraries(box_spread_tests PRIVATE ${UUID_LIBRARY})
    target_link_libraries(box_spread_tws_tests PRIVATE ${UUID_LIBRARY})
endif()

# Link with the same object files as the main executable
# (This avoids code duplication)
# Note: When building from repo root, use native/ prefix
set(_native_src_dir "${CMAKE_SOURCE_DIR}/native/src")
if(NOT EXISTS "${_native_src_dir}")
    # Building from native/ directory directly
    set(_native_src_dir "${CMAKE_SOURCE_DIR}/src")
endif()

target_sources(box_spread_core_tests PRIVATE
    ${_native_src_dir}/config_manager.cpp
    ${_native_src_dir}/path_validator.cpp
    ${_native_src_dir}/market_hours.cpp
    ${_native_src_dir}/rate_limiter.cpp
    ${_native_src_dir}/convexity_calculator.cpp
    ${_native_src_dir}/asset_relationship.cpp
    ${_native_src_dir}/collateral_valuator.cpp
    ${_native_src_dir}/financing_instrument_registry.cpp
)

target_sources(box_spread_tws_tests PRIVATE
    ${_native_src_dir}/config_manager.cpp
    ${_native_src_dir}/path_validator.cpp
    ${_native_src_dir}/types.cpp
    ${_native_src_dir}/tws_conversions.cpp
    ${_native_src_dir}/tws_connection.cpp
    ${_native_src_dir}/tws_market_data.cpp
    ${_native_src_dir}/tws_orders.cpp
    ${_native_src_dir}/tws_positions.cpp
    ${_native_src_dir}/tws_contracts.cpp
    ${_native_src_dir}/tws_error_codes.cpp
    ${_native_src_dir}/connection_utils.cpp
    ${_native_src_dir}/pcap_capture.cpp
    ${_native_src_dir}/tws_client.cpp
    ${_native_src_dir}/order_manager.cpp
)

target_sources(box_spread_tests PRIVATE
    ${_native_src_dir}/config_manager.cpp
    ${_native_src_dir}/path_validator.cpp
    ${_native_src_dir}/types.cpp
    ${_native_src_dir}/tws_conversions.cpp
    ${_native_src_dir}/tws_connection.cpp
    ${_native_src_dir}/tws_market_data.cpp
    ${_native_src_dir}/tws_orders.cpp
    ${_native_src_dir}/tws_positions.cpp
    ${_native_src_dir}/tws_contracts.cpp
    ${_native_src_dir}/tws_error_codes.cpp
    ${_native_src_dir}/connection_utils.cpp
    ${_native_src_dir}/pcap_capture.cpp
    ${_native_src_dir}/tws_client.cpp
    ${_native_src_dir}/market_hours.cpp
    ${_native_src_dir}/rate_limiter.cpp
    ${_native_src_dir}/hedge_manager.cpp
    ${_native_src_dir}/greeks_calculator.cpp
    ${_native_src_dir}/margin_calculator.cpp
    ${_native_src_dir}/nats_client.cpp
    ${_native_src_dir}/cache_client.cpp
    ${_native_src_dir}/strategies/box_spread/box_spread_bag.cpp
    ${_native_src_dir}/convexity_calculator.cpp
    ${_native_src_dir}/asset_relationship.cpp
    ${_native_src_dir}/collateral_valuator.cpp
    ${_native_src_dir}/financing_instrument_registry.cpp
)

# Conditionally include local source files only if NOT using extracted library
if(NOT USE_BOX_SPREAD_CPP_LIB)
    target_sources(box_spread_tests PRIVATE
        ${_native_src_dir}/option_chain.cpp
        ${_native_src_dir}/strategies/box_spread/box_spread_strategy.cpp
        ${_native_src_dir}/order_manager.cpp
        ${_native_src_dir}/risk_calculator.cpp
        ${_native_src_dir}/risk_calculator_var.cpp
        ${_native_src_dir}/risk_calculator_sizing.cpp
        ${_native_src_dir}/risk_calculator_stats.cpp
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
catch_discover_tests(box_spread_core_tests)
catch_discover_tests(box_spread_tests)
catch_discover_tests(box_spread_tws_tests)

# ============================================================================
# Test Target
# ============================================================================

add_custom_target(run_core_tests
    COMMAND box_spread_core_tests
    DEPENDS box_spread_core_tests
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Running core tests..."
)

add_custom_target(run_tws_tests
    COMMAND box_spread_tws_tests
    DEPENDS box_spread_tws_tests
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Running TWS tests..."
)

add_custom_target(run_tests
    COMMAND box_spread_core_tests
    COMMAND box_spread_tws_tests
    COMMAND box_spread_tests
    DEPENDS box_spread_core_tests box_spread_tws_tests box_spread_tests
    WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
    COMMENT "Running tests..."
)
