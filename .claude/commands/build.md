---
description: Build the project (debug mode via Justfile)
---

Build the project and report success/failure:

```bash
ninja -C build
```

If the build directory doesn't exist, configure first:

```bash
cmake -S . -B build -G Ninja -DCMAKE_BUILD_TYPE=Debug && ninja -C build
```

Report: build success/failure, any errors or warnings.
