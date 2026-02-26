Build the project in debug mode. If the build directory doesn't exist, configure CMake first.

RUN test -d build || cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug
RUN ninja -C build
