{ self, pkgs, crane, msrvRust, buildInputs, nativeBuildInputs }:
let
  craneLib = (crane.mkLib pkgs).overrideToolchain (p: msrvRust);
  src = ../..;
  doCheck = false;
in
rec {
  default = neuraldisk-gui-wayland;
  neuraldisk-gui = let
    cargoToml = "${self}/../../neuraldisk_gui/Cargo.toml";
    cargoTomlConfig = builtins.fromTOML (builtins.readFile cargoToml);
    version = cargoTomlConfig.package.version;
  in
  craneLib.buildPackage {
    inherit version src cargoToml buildInputs nativeBuildInputs doCheck;
    name = "neuraldisk-gui";
    cargoExtraArgs = "--bin neuraldisk_gui";
    cargoArtifacts = craneLib.buildDepsOnly {
      inherit version src cargoToml buildInputs nativeBuildInputs doCheck;
      name = "neuraldisk-gui";
      cargoExtraArgs  = "--bin neuraldisk_gui";
    };
  };
  wrapped-neuraldisk-gui = pkgs.writeShellScriptBin "wrapped-neuraldisk-gui" ''
    export GSETTINGS_SCHEMA_DIR ="${pkgs.gtk4}/share/gsettings-schemas/gtk4-${pkgs.gtk4.version}/glib-2.0/schemas";
    exec ${neuraldisk-gui}/bin/neuraldisk_gui "$@"
  '';
  neuraldisk-gui-wayland = let
    cargoToml = "${self}/../../neuraldisk_gui/Cargo.toml";
    cargoTomlConfig = builtins.fromTOML (builtins.readFile cargoToml);
    version = cargoTomlConfig.package.version;
    waylandBuildInputs = buildInputs ++ [ pkgs.wayland ];
  in
  craneLib.buildPackage {
    inherit version src cargoToml nativeBuildInputs doCheck;
    buildInputs = waylandBuildInputs;
    name = "neuraldisk-gui";
    cargoExtraArgs = "--bin neuraldisk_gui";
    cargoArtifacts = craneLib.buildDepsOnly {
      inherit version src cargoToml nativeBuildInputs doCheck;
      name = "neuraldisk-gui";
      cargoExtraArgs  = "--bin neuraldisk_gui";
    };
  };
  neuraldisk-cli = let
    cargoToml = "${self}/../../neuraldisk_cli/Cargo.toml";
    cargoTomlConfig = builtins.fromTOML (builtins.readFile cargoToml);
    version = cargoTomlConfig.package.version;
  in
  craneLib.buildPackage {
    inherit version src cargoToml doCheck;
    buildInputs = [];
    nativeBuildInputs = [];
    name = "neuraldisk-cli";
    cargoExtraArgs = "--bin neuraldisk_cli";
    cargoArtifacts = craneLib.buildDepsOnly {
      inherit version src cargoToml buildInputs nativeBuildInputs doCheck;
      name = "neuraldisk-cli";
      cargoExtraArgs  = "--bin neuraldisk_cli";
    };
  };
}
