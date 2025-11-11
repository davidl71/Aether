cmake_minimum_required(VERSION 3.15)

file(GLOB SOURCES "*.cpp")
file(GLOB HEADERS "*.h")
set(twsapi_VERSION 9.79.02)
add_library(twsapi SHARED ${SOURCES})

# Get project root for Intel Decimal library
get_filename_component(PROJECT_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../../../../../../.." ABSOLUTE)
set(INTEL_BID_LIB "${PROJECT_ROOT}/native/third_party/IntelRDFPMathLib20U2/LIBRARY/libbid.a")

# On Windows (including MinGW), link against Winsock 2
if( WIN32 )
    target_link_libraries( twsapi
        PUBLIC
        ws2_32
    )
endif()

# Link Intel Decimal library and other dependencies
target_link_libraries( twsapi
    PUBLIC
        "${INTEL_BID_LIB}"
        "/usr/local/lib/libprotobuf.dylib"
        "-lz"
        "-lpthread"
)

# Link Abseil libraries if available (required by modern protobuf)
file(GLOB ABSEIL_LIBS "/usr/local/lib/libabsl*.dylib")
if(ABSEIL_LIBS)
    target_link_libraries(twsapi PUBLIC ${ABSEIL_LIBS})
endif()

if( MSVC )
    target_compile_options( twsapi PRIVATE /W3 )
else()
    target_compile_options( twsapi PRIVATE -Wall -Wextra -pedantic -Wno-unused-parameter -Wno-deprecated-declarations )
endif()

set_target_properties(twsapi
    PROPERTIES
    PREFIX "lib"
    DEBUG_POSTFIX "d"
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib"
    PUBLIC_HEADER "${HEADERS}"
)
install(TARGETS twsapi EXPORT twsapi-targets PUBLIC_HEADER DESTINATION include/twsapi LIBRARY DESTINATION lib)