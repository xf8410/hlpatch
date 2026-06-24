cmake_minimum_required(VERSION 3.18)
project(umahook VERSION 0.1 LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
include(FetchContent)
FetchContent_Declare(dobby
GIT_REPOSITORY https://github.com/jmpews/Dobby.git
GIT_TAG master GIT_SHALLOW TRUE)
set(DOBBY_DEBUG OFF CACHE BOOL "" FORCE)
set(DOBBY_BUILD_EXAMPLE OFF CACHE BOOL "" FORCE)
set(DOBBY_BUILD_TEST OFF CACHE BOOL "" FORCE)
set(Plugin.SymbolResolver OFF CACHE BOOL "" FORCE)
set(Plugin.ImportTableReplace OFF CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(dobby)
set(UMAHOOK_SOURCES
src/main.cpp src/il2cpp_hook.cpp
src/game_hooks.cpp src/http_server.cpp)
add_library(main SHARED ${UMAHOOK_SOURCES})
target_include_directories(main PRIVATE
${CMAKE_CURRENT_SOURCE_DIR}/src
${CMAKE_CURRENT_SOURCE_DIR}/include)
target_link_libraries(main dobby_static log dl)
target_compile_options(main PRIVATE -Wall
-Wno-unused-parameter
-Wno-missing-field-initializers
-fno-rtti -fno-exceptions -fvisibility=hidden)
target_link_options(main PRIVATE
-Wl,--version-script=${CMAKE_CURRENT_SOURCE_DIR}/export.map)
set_target_properties(main PROPERTIES OUTPUT_NAME "main" PREFIX "lib")
