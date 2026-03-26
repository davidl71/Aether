{
  description = "Synthetic Financing Platform — Nix dev shell (C++, Python, Rust, Node). Use with: nix develop";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";

  outputs = { self, nixpkgs }: let
    supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    pkgsFor = system: import nixpkgs {
      inherit system;
      config.allowUnfree = true;
    };
  in {
    devShells = forAllSystems (system: let
      pkgs = pkgsFor system;
    in {
      default = pkgs.mkShell {
        name = "ib-box-spread-dev";

        buildInputs = with pkgs; [
          # C++ build (stdenv.cc = clang on macOS, gcc on Linux)
          cmake
          ninja
          stdenv.cc
          boost
          protobuf
          abseil-cpp
          curl
          pkg-config
          # Python (project prefers uv; uv is below)
          python3
          uv
          # Rust (agents/)
          rustc
          cargo
          # Node (scripts/)
          nodejs_22
          # Lint / scripts
          jq
          shellcheck
          cppcheck
          # Git / DB
          git
          sqlite
        ];

        shellHook = ''
          echo "ib_box_spread dev shell — cmake, ninja, python, uv, cargo, node available."
          echo "Build: cmake -S . -B build -G Ninja && ninja -C build"
          echo "Tests: ctest --test-dir build --output-on-failure"
          echo "Python: uv run --with pytest pytest native/tests/python/"
        '';
      };
    });
  };
}
