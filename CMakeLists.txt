cmake_minimum_required(VERSION 3.21)

project(BlockyBuild LANGUAGES CXX)

# -----------------------------
# C++ / MSVC configuration
# -----------------------------
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

if (MSVC)
    # Godot is usually built with /MD; adjust if your static lib requires /MT
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
    add_compile_options(/EHsc)
endif()

# -----------------------------
# Paths (adjust to your tree)
# -----------------------------
set(GODOT_ROOT ${CMAKE_SOURCE_DIR}/external/godot)
set(GODOT_INCLUDE_DIR ${GODOT_ROOT}/bin/include)
set(GODOT_LIB_DIR ${GODOT_ROOT}/bin)

# -----------------------------
# Executable
# -----------------------------
add_executable(BlockyBuild
    src/main.cpp
)

# -----------------------------
# Include paths
# -----------------------------
target_include_directories(BlockyBuild PRIVATE
    ${GODOT_INCLUDE_DIR}
)

# -----------------------------
# Link against Godot static library
# -----------------------------
target_link_libraries(BlockyBuild PRIVATE
    ${GODOT_LIB_DIR}/godot.windows.template_release.x86_64.nothreads.lib
)

# -----------------------------
# Optional: define any static flags or macros Godot requires
# -----------------------------
# target_compile_definitions(BlockyBuild PRIVATE STATIC_GODOT_BUILD)

# No DLL copy step needed — fully static
