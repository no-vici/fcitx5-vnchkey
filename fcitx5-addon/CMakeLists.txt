cmake_minimum_required(VERSION 3.16)
project(fcitx5-vnchkey-addon VERSION 0.3.3 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

find_package(ECM 1.0.0 QUIET NO_MODULE)
if(ECM_FOUND)
    set(CMAKE_MODULE_PATH ${ECM_MODULE_PATH} ${CMAKE_MODULE_PATH})
endif()

find_package(Fcitx5Core 5.0.0 REQUIRED)
find_package(Fcitx5Utils 5.0.0 REQUIRED)

include(GNUInstallDirs)

if(NOT DEFINED FCITX_INSTALL_PKGDATADIR)
    set(FCITX_INSTALL_PKGDATADIR "${CMAKE_INSTALL_DATADIR}/fcitx5")
endif()
if(NOT DEFINED FCITX_INSTALL_LIBDIR)
    set(FCITX_INSTALL_LIBDIR "${CMAKE_INSTALL_LIBDIR}/fcitx5")
endif()

# Locate the compiled Rust FFI library
set(RUST_SEARCH_DIRS
    "${RUST_TARGET_DIR}"
    "${CMAKE_CURRENT_SOURCE_DIR}/../target/release"
    "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug"
    "$ENV{CARGO_TARGET_DIR}/release"
    "/usr/lib"
    "/usr/local/lib"
)

find_library(RUST_FFI_LIB
    NAMES fcitx5_vnchkey_ffi
    PATHS ${RUST_SEARCH_DIRS}
    NO_DEFAULT_PATH
)

if(NOT RUST_FFI_LIB)
    foreach(d IN LISTS RUST_SEARCH_DIRS)
        if(EXISTS "${d}/libfcitx5_vnchkey_ffi.so")
            set(RUST_FFI_LIB "${d}/libfcitx5_vnchkey_ffi.so")
            break()
        endif()
    endforeach()
endif()

if(NOT RUST_FFI_LIB)
    find_library(RUST_FFI_LIB NAMES fcitx5_vnchkey_ffi)
endif()

if(NOT RUST_FFI_LIB)
    message(FATAL_ERROR "Could not find libfcitx5_vnchkey_ffi.so. Please run 'cargo build --release' first!")
endif()

add_library(fcitx5-vnchkey-addon MODULE
    src/vnchkey_addon.cpp
)

target_include_directories(fcitx5-vnchkey-addon PRIVATE
    src
)

target_link_libraries(fcitx5-vnchkey-addon PRIVATE
    Fcitx5::Core
    Fcitx5::Utils
    ${RUST_FFI_LIB}
)

set_target_properties(fcitx5-vnchkey-addon PROPERTIES
    PREFIX ""
    OUTPUT_NAME "fcitx5-vnchkey-addon"
)

# Install addon module
install(TARGETS fcitx5-vnchkey-addon DESTINATION "${FCITX_INSTALL_LIBDIR}")

# Install Fcitx5 addon declaration
install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/../data/addon/vnchkey.conf"
    DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon"
)

# Install Fcitx5 input method entry
install(FILES
    "${CMAKE_CURRENT_SOURCE_DIR}/../data/inputmethod/vnchkey.conf"
    DESTINATION "${FCITX_INSTALL_PKGDATADIR}/inputmethod"
)

# Install icons (scaled to 24x24 / 22x22 matching lilypad standard)
install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../data/icons/hicolor"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/icons"
)
install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../data/icons/breeze"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/icons"
)
install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../data/icons/breeze-dark"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/icons"
)

# Install Native Rust Settings GUI application
if(EXISTS "${RUST_TARGET_DIR}/fcitx5-vnchkey-settings")
    install(PROGRAMS
        "${RUST_TARGET_DIR}/fcitx5-vnchkey-settings"
        DESTINATION "${CMAKE_INSTALL_BINDIR}"
    )
elseif(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/../bin/fcitx5-vnchkey-settings")
    install(PROGRAMS
        "${CMAKE_CURRENT_SOURCE_DIR}/../bin/fcitx5-vnchkey-settings"
        DESTINATION "${CMAKE_INSTALL_BINDIR}"
    )
endif()

install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/../data/org.fcitx.Fcitx5.Addon.Vnchkey.Settings.desktop"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/applications"
)


