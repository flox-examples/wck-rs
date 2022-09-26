{
  self,
  lib,
  rustPlatform,
  hostPlatform,
  # you can add imports here
  openssl,
  pkg-config,
  libiconv,
  darwin,
}:
rustPlatform.buildRustPackage rec {
  pname = "wck-rs";
  version = "0.0.0";
  src = self; # + "/src";

  cargoLock = {
    lockFile = self + "/Cargo.lock";
    # The hash of each dependency that uses a git source must be specified.
    # The hash can be found by setting it to lib.fakeSha256
    # as shown below and running flox build.
    # The build will fail but output the expected sha, which can then be added
    # here
    # the expected error will look like this: 
    #
    #     error: hash mismatch in fixed-output derivation '/nix/store/XXX-dependency-githash.drv':
    #         specified: sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
    #            got:    sha256-M+otd/fsECgT2IRoMwiDOhxMqVGnCyYr7NtKFKuhVNA=
    #
    # If you follow a branch or tag, you might hit this error again in the 
    # future as a the source moves forward.
    # It is advisable to therefore specify exact revisions or known stable tags.
    outputHashes = {
      #   "dependency-0.0.0" = lib.fakeSha256; 
      "clap-3.2.22" = "sha256-M+otd/fsECgT2IRoMwiDOhxMqVGnCyYr7NtKFKuhVNA";
    };
  };



  # Non-Rust runtime dependencies (most likely libraries) of your project can 
  # be added in buildInputs.
  # Make sure to import any additional dependencies above
  buildInputs =
    [
      openssl.dev
    ]
    # Platform specific dependencies can be added as well
    # For MacOS
    ++ lib.optional hostPlatform.isDarwin [
      # If you're getting linker errors about missing libraries, you can add
      # them here
      libiconv
      # If you're getting linker errors about missing frameworks, you can add
      # apple frameworks here
      darwin.apple_sdk.frameworks.Security
    ]
    # and Linux
    ++ lib.optional hostPlatform.isLinux [ ]
    ;


  # Add runtime dependencies required by packages that depend on this package
  # to propagatedBuildInputs.
  propagatedBuildInputs = [];

  # Add buildtime dependencies (not required at runtime) to nativeBuildInputs.
  nativeBuildInputs = [
    pkg-config # for openssl
  ];

  RUST_SRC_PATH = lib.optionalString lib.inNixShell "${rustPlatform.rustLibSrc}";

}
