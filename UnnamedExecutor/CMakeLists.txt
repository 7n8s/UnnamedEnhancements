cmake_minimum_required(VERSION 3.24)
project(UnnamedExecutor VERSION 0.2.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTORCC ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Widgets)

include(FetchContent)
set(LUAU_BUILD_CLI OFF CACHE BOOL "" FORCE)
set(LUAU_BUILD_TESTS OFF CACHE BOOL "" FORCE)
set(LUAU_BUILD_WEB OFF CACHE BOOL "" FORCE)
FetchContent_Declare(
    Luau
    GIT_REPOSITORY https://github.com/luau-lang/luau.git
    GIT_TAG 0.676
    GIT_SHALLOW TRUE
)
FetchContent_MakeAvailable(Luau)

qt_add_executable(UnnamedExecutor WIN32
    src/main.cpp
    src/MainWindow.h
    src/MainWindow.cpp
    src/CodeEditor.h
    src/CodeEditor.cpp
    src/LuauHighlighter.h
    src/LuauHighlighter.cpp
    src/LuauRunner.h
    src/LuauRunner.cpp
)
target_link_libraries(UnnamedExecutor PRIVATE Qt6::Widgets Luau.VM Luau.Compiler)
target_include_directories(UnnamedExecutor PRIVATE ${luau_SOURCE_DIR}/VM/include)
set_target_properties(UnnamedExecutor PROPERTIES OUTPUT_NAME "UnnamedExecutor")
