cmake_minimum_required(VERSION 3.16)
project(mdtoview VERSION 0.1.0 LANGUAGES NONE)

# ── Options ───────────────────────────────────────────────────────────────────
option(CARGO_OFFLINE  "Build cargo with --offline (uses vendored deps)" ON)
option(CARGO_RELEASE  "Build in release mode"                           ON)

set(CARGO_TARGET "" CACHE STRING "Rust target triple (empty = native)")

# ── Locate cargo ─────────────────────────────────────────────────────────────
find_program(CARGO_EXECUTABLE cargo REQUIRED
    HINTS "$ENV{HOME}/.cargo/bin" /usr/local/bin /usr/bin
)
message(STATUS "Using cargo: ${CARGO_EXECUTABLE}")

# ── Build flags ───────────────────────────────────────────────────────────────
set(CARGO_FLAGS "")

if(CARGO_RELEASE)
    list(APPEND CARGO_FLAGS "--release")
    set(CARGO_PROFILE "release")
else()
    set(CARGO_PROFILE "debug")
endif()

if(CARGO_OFFLINE)
    list(APPEND CARGO_FLAGS "--offline")
endif()

if(CARGO_TARGET)
    list(APPEND CARGO_FLAGS "--target" "${CARGO_TARGET}")
    set(BINARY_SUBDIR "${CARGO_TARGET}/${CARGO_PROFILE}")
else()
    set(BINARY_SUBDIR "${CARGO_PROFILE}")
endif()

# ── Custom build target ───────────────────────────────────────────────────────
add_custom_target(mdtoview_cargo ALL
    COMMAND ${CARGO_EXECUTABLE} build ${CARGO_FLAGS}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
    COMMENT "Building mdtoview with cargo (profile=${CARGO_PROFILE})"
    VERBATIM
)

# ── Install ───────────────────────────────────────────────────────────────────
set(BINARY_PATH "${CMAKE_SOURCE_DIR}/target/${BINARY_SUBDIR}/mdtoview")

install(
    PROGRAMS    ${BINARY_PATH}
    DESTINATION ${CMAKE_INSTALL_PREFIX}/bin
    COMPONENT   Runtime
)

install(
    FILES       ${CMAKE_SOURCE_DIR}/packaging/mdtoview.desktop
    DESTINATION ${CMAKE_INSTALL_PREFIX}/share/applications
    OPTIONAL
)

# ── Summary ───────────────────────────────────────────────────────────────────
message(STATUS "")
message(STATUS "  mdtoview ${PROJECT_VERSION}")
message(STATUS "  Profile  : ${CARGO_PROFILE}")
message(STATUS "  Offline  : ${CARGO_OFFLINE}  (vendor/ dir used when ON)")
message(STATUS "  Target   : ${CARGO_TARGET} (empty = native)")
message(STATUS "  Install  : ${CMAKE_INSTALL_PREFIX}/bin")
message(STATUS "")
