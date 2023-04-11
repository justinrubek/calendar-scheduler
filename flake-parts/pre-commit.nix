{
  inputs,
  self,
  ...
}: {
  perSystem = {
    pkgs,
    self',
    ...
  }: {
    pre-commit = {
      check.enable = false;

      settings = {
        src = ../.;
        hooks = {
          alejandra.enable = true;
          rustfmt.enable = true;
        };
      };
    };
  };
}
