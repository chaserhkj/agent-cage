{ rustPlatform }: rustPlatform.buildRustPackage rec {
  name = "agent-cage";
  pname = name;
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
  meta.mainProgram = pname;
}