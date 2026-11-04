cmake_minimum_required(VERSION 3.16)
project(fcitx5-vnchkey-addon VERSION 0.2.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

find_package(ECM 1.0.0 REQUIRED NO_MODULE)
set(CMAKE_MODULE_PATH ${ECM_MODULE_PATH} ${CMAKE_MODULE_PATH})

find_package(Fcitx5Core 5.0.0 REQUIRED)

include(GNUInstallDirs)

# Locate the compiled Rust FFI library (from target/release/)
set(RUST_TARGET_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../target/release")
find_library(RUST_FFI_LIB
    NAMES fcitx5_vnchkey_ffi
    PATHS "${RUST_TARGET_DIR}"
    NO_DEFAULT_PATH
)

if(NOT RUST_FFI_LIB)
    find_library(RUST_FFI_LIB NAMES fcitx5_vnchkey_ffi)
endif()

add_library(fcitx5-vnchkey-addon MODULE
    src/vnchkey_addon.cpp
)

target_include_directories(fcitx5-vnchkey-addon PRIVATE
    src
)

target_link_libraries(fcitx5-vnchkey-addon PRIVATE
    Fcitx5::Core
    ${RUST_FFI_LIB}
)

set_target_properties(fcitx5-vnchkey-addon PROPERTIES
    PREFIX ""
    OUTPUT_NAME "fcitx5-vnchkey-addon"
)

# Install addon module
install(TARGETS fcitx5-vnchkey-addon DESTINATION "${FCITX_INSTALL_LIBDIR}/fcitx5")

# Install Fcitx5 addon declaration
install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/../data/addon/vnchkey.conf"
    DESTINATION "${FCITX_INSTALL_PKGDATADIR}/addon"
)

# Install Fcitx5 input method entries (Telex & VNI)
install(FILES
    "${CMAKE_CURRENT_SOURCE_DIR}/../data/inputmethod/vnchkey-telex.conf"
    "${CMAKE_CURRENT_SOURCE_DIR}/../data/inputmethod/vnchkey-vni.conf"
    DESTINATION "${FCITX_INSTALL_PKGDATADIR}/inputmethod"
)

# Install icon
install(FILES "${CMAKE_CURRENT_SOURCE_DIR}/../data/icons/vnchkey.png"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/icons/hicolor/128x128/apps"
)

