
with import <nixpkgs> { };

mkShell {
  nativeBuildInputs = [
    direnv
    rustc
    cargo
    python3
    python3Packages.beautifulsoup4
    python3Packages.requests
  ];

  NIX_ENFORCE_PURITY = true;
}