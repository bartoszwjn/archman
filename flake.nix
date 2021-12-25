{
  description = "A configuration utility for my specific Arch Linux setup";

  outputs = { self, nixpkgs }: {
    defaultPackage.x86_64-linux =
      import ./. { pkgs = import nixpkgs { system = "x86_64-linux"; }; };
  };
}
