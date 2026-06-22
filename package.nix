{ buildRustPackage }: buildRustPackage rec {
  pname = "agent-cage";
  src = ./.;
  cargoLock.localFile = ./Cargo.lock;
  meta.mainProgram = pname;
}