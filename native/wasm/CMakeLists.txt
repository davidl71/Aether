# WASM Build Configuration for Box Spread Calculations
# Compiles C++ business logic to WebAssembly for use in React web app

cmake_minimum_required(VERSION 3.21)
project(ib_box_spread_wasm)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Emscripten compiler settings
if(NOT DEFINED EMSCRIPTEN_ROOT_PATH)
  message(FATAL_ERROR "Emscripten not found. Please install and activate emsdk first.")
endif()

set(CMAKE_C_COMPILER "${EMSCRIPTEN_ROOT_PATH}/emcc")
set(CMAKE_CXX_COMPILER "${EMSCRIPTEN_ROOT_PATH}/em++")
set(CMAKE_EXECUTABLE_SUFFIX ".js")

# WASM-specific compilation flags (compile-time)
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -O3")
set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s DISABLE_EXCEPTION_CATCHING=0")

# WASM-specific linker flags (link-time)
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -s WASM=1")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -s EXPORT_ES6=1")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -s MODULARIZE=1")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -s EXPORT_NAME=createBoxSpreadModule")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -s ALLOW_MEMORY_GROWTH=1")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -s EXPORTED_FUNCTIONS='[\"_malloc\",\"_free\"]'")
set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} -s EXPORTED_RUNTIME_METHODS='[\"ccall\",\"cwrap\",\"UTF8ToString\",\"stringToUTF8\"]'")
# Note: USE_ES6_IMPORT_META flag removed - no longer supported in Emscripten 4.0+

# Dependencies
include(FetchContent)

# nlohmann/json (header-only, WASM-compatible)
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

fetchcontent_makeavailable(nlohmann_json)

# Include directories
include_directories(
  ${CMAKE_CURRENT_SOURCE_DIR}/../include
  ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Source files - calculation modules only (no TWS/network dependencies)
set(WASM_SOURCES
  ${CMAKE_CURRENT_SOURCE_DIR}/../src/box_spread_strategy.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/../src/risk_calculator.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/../src/option_chain.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/../src/tui_converter.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/wasm_bindings.cpp
  ${CMAKE_CURRENT_SOURCE_DIR}/src/wasm_exports.cpp
)

# Headers
set(WASM_HEADERS
  ${CMAKE_CURRENT_SOURCE_DIR}/../include/box_spread_strategy.h
  ${CMAKE_CURRENT_SOURCE_DIR}/../include/risk_calculator.h
  ${CMAKE_CURRENT_SOURCE_DIR}/../include/option_chain.h
  ${CMAKE_CURRENT_SOURCE_DIR}/../include/tui_converter.h
  ${CMAKE_CURRENT_SOURCE_DIR}/../include/types.h
  ${CMAKE_CURRENT_SOURCE_DIR}/include/wasm_types.h
)

# Create WASM module
add_executable(box_spread_wasm ${WASM_SOURCES} ${WASM_HEADERS})

# Link dependencies
target_link_libraries(box_spread_wasm
  PRIVATE
    nlohmann_json::nlohmann_json
)

# Note: We avoid heavy dependencies like spdlog in WASM
# Logging should be stubbed or removed for WASM builds
# TODO: Remove or stub spdlog includes in source files for WASM build

# Installation
install(TARGETS box_spread_wasm
  DESTINATION ${CMAKE_CURRENT_SOURCE_DIR}/../../web/public/wasm
)
