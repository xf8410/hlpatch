cmake_minimum_required(VERSION 3.18)
project(umahook VERSION 0.1 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# ===== Dobby (shared library) =====
set(DOBBY_LIB "${CMAKE_CURRENT_SOURCE_DIR}/libs/libdobby.so")
set(DOBBY_INC "${CMAKE_CURRENT_SOURCE_DIR}/libs/include")

# ===== Hook source files =====
set(UMAHOOK_SOURCES
    src/main.cpp
    src/il2cpp_hook.cpp
    src/game_hooks.cpp
    src/http_server.cpp
)

# ===== Build shared library =====
add_library(main SHARED ${UMAHOOK_SOURCES})

# ===== Include paths =====
target_include_directories(main PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${DOBBY_INC}
)

# ===== Link libraries =====
target_link_libraries(main
    ${DOBBY_LIB}
    log
    dl
)

# ===== Compile options =====
target_compile_options(main PRIVATE
    -Wall
    -Wno-unused-parameter
    -Wno-missing-field-initializers
    -fno-rtti
    -fvisibility=hidden
)

# Export JNI_OnLoad
target_link_options(main PRIVATE
    -Wl,--version-script=${CMAKE_CURRENT_SOURCE_DIR}/export.map
)

# Output libmain.so
set_target_properties(main PROPERTIES
    OUTPUT_NAME "main"
    PREFIX "lib"
)
