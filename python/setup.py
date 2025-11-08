#!/usr/bin/env python3
"""
setup.py - Main package setup for IB Box Spread Generator Python integration
"""
from setuptools import setup, find_packages
from pathlib import Path

# Read version from project or use default
VERSION = "1.0.0"

# Read long description from README if available
readme_file = Path(__file__).parent.parent / "README.md"
long_description = ""
if readme_file.exists():
    long_description = readme_file.read_text()

setup(
    name="ib-box-spread-generator",
    version=VERSION,
    description="IB Box Spread Generator - Python integration with NautilusTrader",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="IB Box Spread Generator Team",
    author_email="",
    url="https://github.com/yourusername/ib-box-spread-generator",
    packages=find_packages(where=".", exclude=["tests", "tests.*", "bindings"]),
    py_modules=["nautilus_strategy", "config_adapter"],
    python_requires=">=3.11",
    install_requires=[
        "nautilus_trader>=2.0.0",
        "numpy>=1.24.0",
    ],
    extras_require={
        "dev": [
            "pytest>=7.4.0",
            "pytest-cov>=4.1.0",
            "cython>=3.0.0",
        ],
        "all": [
            "cython>=3.0.0",
            "pytest>=7.4.0",
            "pytest-cov>=4.1.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "ib-box-spread-nautilus=nautilus_strategy:main",
        ],
    },
    package_dir={"": "."},
    package_data={
        "": ["*.pxd", "*.pyx"],
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Financial and Insurance Industry",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: 3.13",
        "Programming Language :: C++",
        "Topic :: Office/Business :: Financial :: Investment",
    ],
    zip_safe=False,
)

