#!/usr/bin/env python3
"""
setup.py - Build script for Cython bindings.

Requires a prior CMake build so build/_deps (nlohmann, spdlog, Eigen) and
build/lib (QuantLib, nlopt) exist. At import time the .so may still resolve
symbols from other native code (e.g. TWS client); see python/bindings/README.md.
"""
import os
import sys
from pathlib import Path

from setuptools import setup, Extension
from Cython.Build import cythonize
import numpy

# Get the project root and native paths (C++ sources are in native/)
project_root = Path(__file__).resolve().parent.parent.parent
native_dir = project_root / "native"
src_dir = native_dir / "src"
include_dir = native_dir / "include"

# Find nlohmann/json, spdlog, and Eigen from CMake FetchContent (build/_deps/...)
def _find_fetchcontent_includes():
    build_dir = project_root / "build"
    if not build_dir.is_dir():
        return []
    result = []
    # nlohmann: .../single_include so that #include <nlohmann/json.hpp> works
    single = build_dir / "_deps" / "nlohmann_json-src" / "single_include"
    if single.is_dir():
        result.append(str(single))
    else:
        for child in build_dir.iterdir():
            if child.is_dir():
                single = child / "_deps" / "nlohmann_json-src" / "single_include"
                if single.is_dir():
                    result.append(str(single))
                    break
    # spdlog: .../include so that #include <spdlog/spdlog.h> works
    spdlog_inc = build_dir / "_deps" / "spdlog-src" / "include"
    if spdlog_inc.is_dir():
        result.append(str(spdlog_inc))
    else:
        for child in build_dir.iterdir():
            if child.is_dir():
                spdlog_inc = child / "_deps" / "spdlog-src" / "include"
                if spdlog_inc.is_dir():
                    result.append(str(spdlog_inc))
                    break
    # Eigen: .../eigen3-src so that #include <Eigen/Dense> works
    eigen_inc = build_dir / "_deps" / "eigen3-src"
    if eigen_inc.is_dir():
        result.append(str(eigen_inc))
    else:
        for child in build_dir.iterdir():
            if child.is_dir():
                eigen_inc = child / "_deps" / "eigen3-src"
                if eigen_inc.is_dir():
                    result.append(str(eigen_inc))
                    break
    return result

include_dirs_list = [str(include_dir), numpy.get_include()]
fetch_includes = _find_fetchcontent_includes()
if fetch_includes:
    include_dirs_list.extend(fetch_includes)
else:
    print("Warning: nlohmann/json, spdlog, or Eigen not found under build/_deps. Run CMake build first.", file=sys.stderr)

# Paths relative to setup.py dir so setuptools invokes compiler with correct paths
bindings_dir = Path(__file__).resolve().parent
def rel_to_setup(p: Path) -> str:
    try:
        return str(p.relative_to(bindings_dir))
    except ValueError:
        return str(p)
sources_rel = [
    "box_spread_bindings.pyx",
    rel_to_setup(src_dir / "strategies" / "box_spread" / "box_spread_strategy.cpp"),
    rel_to_setup(src_dir / "risk_calculator.cpp"),
    rel_to_setup(src_dir / "option_chain.cpp"),
    rel_to_setup(src_dir / "config_manager.cpp"),
    rel_to_setup(src_dir / "market_hours.cpp"),
]

# Compiler flags
compile_args = ["-std=c++20", "-O3"]
if os.name == "posix":
    compile_args.extend(["-fPIC"])
if sys.platform == "darwin":
    import subprocess
    try:
        sdk = subprocess.run(["xcrun", "--show-sdk-path"], capture_output=True, text=True, check=True)
        sdk_path = sdk.stdout.strip()
        cxx_include = f"{sdk_path}/usr/include/c++/v1"
        if os.path.isdir(cxx_include):
            compile_args.extend(["-isystem", cxx_include])
    except (subprocess.CalledProcessError, FileNotFoundError):
        pass

# Linker flags and libs (QuantLib, NLopt used by option_chain and risk_calculator)
build_dir = project_root / "build"
library_dirs_list = []
libraries_list = []
lib_dir = build_dir / "lib" if build_dir.is_dir() else None
if lib_dir and lib_dir.is_dir():
    library_dirs_list = [str(lib_dir)]
    # Link against QuantLib and NLopt (built by CMake FetchContent)
    if (lib_dir / "libQuantLib.dylib").exists() or (lib_dir / "libQuantLib.so").exists():
        libraries_list.append("QuantLib")
    if (lib_dir / "libnlopt.dylib").exists() or (lib_dir / "libnlopt.so").exists():
        libraries_list.append("nlopt")

link_args = []
if sys.platform == "darwin" and library_dirs_list:
    for d in library_dirs_list:
        link_args.append(f"-Wl,-rpath,{d}")
elif sys.platform.startswith("linux") and library_dirs_list:
    for d in library_dirs_list:
        link_args.append(f"-Wl,-rpath,{d}")

# Define the extension
extensions = [
    Extension(
        "box_spread_bindings",
        sources=sources_rel,
        include_dirs=include_dirs_list,
        language="c++",
        extra_compile_args=compile_args,
        extra_link_args=link_args,
        library_dirs=library_dirs_list,
        libraries=libraries_list,
    )
]

setup(
    name="box_spread_bindings",
    version="1.0.0",
    description="Cython bindings for IB Box Spread Generator C++ calculations",
    ext_modules=cythonize(
        extensions,
        compiler_directives={
            "language_level": "3",
            "boundscheck": False,
            "wraparound": False,
            "cdivision": True,
        },
    ),
    zip_safe=False,
    python_requires=">=3.11",
    install_requires=[
        "cython>=3.0.0",
        "numpy>=1.24.0",
    ],
)



