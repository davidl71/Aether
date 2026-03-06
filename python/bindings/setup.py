#!/usr/bin/env python3
"""
setup.py - Build script for Cython bindings
"""
from setuptools import setup, Extension
from Cython.Build import cythonize
import numpy
import os
from pathlib import Path

# Get the project root directory
project_root = Path(__file__).parent.parent.parent
src_dir = project_root / "src"
include_dir = project_root / "include"

# Compiler flags
compile_args = ["-std=c++20", "-O3"]
if os.name == "posix":
    compile_args.extend(["-fPIC"])

# Linker flags
link_args = []

# Define the extension
extensions = [
    Extension(
        "box_spread_bindings",
        sources=[
            "box_spread_bindings.pyx",
            str(src_dir / "box_spread_strategy.cpp"),
            str(src_dir / "risk_calculator.cpp"),
            str(src_dir / "option_chain.cpp"),
            str(src_dir / "config_manager.cpp"),
        ],
        include_dirs=[
            str(include_dir),
            numpy.get_include(),
        ],
        language="c++",
        extra_compile_args=compile_args,
        extra_link_args=link_args,
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



