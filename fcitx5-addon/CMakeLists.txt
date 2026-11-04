cmake_minimum_required(VERSION 3.16)
project(fcitx5-vnchkey-addon VERSION 0.2.0 LANGUAGES CXX)

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
if(NOT DEFINED RUST_TARGET_DIR)
    if(DEFINED ENV{CARGO_TARGET_DIR})
        set(RUST_TARGET_DIR "$ENV{CARGO_TARGET_DIR}/release")
    else()
        set(RUST_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../target/release")
    endif()
endif()

find_library(RUST_FFI_LIB
    NAMES fcitx5_vnchkey_ffi
    PATHS "${RUST_TARGET_DIR}"
    NO_DEFAULT_PATH
)

if(NOT RUST_FFI_LIB)
    if(EXISTS "${RUST_TARGET_DIR}/libfcitx5_vnchkey_ffi.so")
        set(RUST_FFI_LIB "${RUST_TARGET_DIR}/libfcitx5_vnchkey_ffi.so")
    else()
        find_library(RUST_FFI_LIB NAMES fcitx5_vnchkey_ffi)
    endif()
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

# Install Fcitx5 input method entries (Auto, Telex, VNI)
install(FILES
    "${CMAKE_CURRENT_SOURCE_DIR}/../data/inputmethod/vnchkey.conf"
    "${CMAKE_CURRENT_SOURCE_DIR}/../data/inputmethod/vnchkey-telex.conf"
    "${CMAKE_CURRENT_SOURCE_DIR}/../data/inputmethod/vnchkey-vni.conf"
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

# Install Settings GUI application
install(PROGRAMS "${CMAKE_CURRENT_SOURCE_DIR}/../bin/vnchkey-settings"
    DESTINATION "${CMAKE_INSTALL_BINDIR}"
)
install(DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}/../settings-gui"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/fcitx5-vnchkey"
)
install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/../data/org.fcitx.Fcitx5.Addon.Vnchkey.Settings.desktop"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/applications"
)


