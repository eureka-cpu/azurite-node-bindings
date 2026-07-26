final: prev:
{
  azurite = prev.azurite.overrideAttrs (_:
    let
      info = builtins.fromJSON (builtins.readFile ./azurite/info.json);
      src =
        let
          src = final.fetchFromGitHub {
            owner = "Azure";
            repo = "Azurite";
            rev = "v${info.version}";
            inherit (info) hash;
          };
        in
        final.runCommand "azurite-${info.version}-src" { } ''
          cp -r ${src} $out
          chmod -R +w $out
          cp ${./azurite/package-lock.json} $out/package-lock.json
        '';
    in
    {
      inherit src;
      inherit (info) version;
      npmDeps = final.fetchNpmDeps {
        inherit src;
        hash = info.npmDepsHash;
      };
      passthru.updateScript = ./azurite/update.sh;
    });
}
