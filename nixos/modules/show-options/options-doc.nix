# Formats NixOS module options for all four azurite services as a pretty, ANSI-coloured string.
{ pkgs }:
let
  lib = pkgs.lib;

  esc = builtins.fromJSON "\"\\u001B\"";
  reset = "${esc}[0m";
  bold = s: "${esc}[1m${s}${reset}";
  dim = s: "${esc}[2m${s}${reset}";
  cyan = s: "${esc}[96m${s}${reset}";
  green = s: "${esc}[32m${s}${reset}";

  # Call the module function with stub args so we can read its declared options
  # without evaluating the config block (which references systemd, etc.).
  moduleOpts = module: serviceName:
    let
      result = import module { config = { }; inherit lib pkgs; };
    in
    result.options.services.${serviceName};

  typeDesc = opt:
    let
      n = opt.type.description or opt.type.name or "unknown";
    in
    if lib.hasPrefix "16 bit unsigned integer" n then "port" else n;

  defaultOf = opt:
    if opt ? defaultText then
      let dt = opt.defaultText;
      in if builtins.isAttrs dt && dt ? text then dt.text else builtins.toString dt
    else if opt ? default then
      let v = opt.default;
      in
      if v == null then "null"
      else if builtins.isBool v then (if v then "true" else "false")
      else if builtins.isInt v then builtins.toString v
      else if builtins.isString v then "\"${v}\""
      else if lib.isDerivation v then "<package>"
      else lib.generators.toPretty { } v
    else "-";

  descOf = opt:
    let d = opt.description or "";
    in if builtins.isString d then d else d.text or "";

  fmtOption = name: opt:
    "  ${bold (cyan name)}  ${dim "[${typeDesc opt}]"}  ${dim "default: ${defaultOf opt}"}\n    ${descOf opt}";

  fmtSection = module: serviceName:
    let
      opts = moduleOpts module serviceName;
      names = lib.naturalSort (lib.attrNames (lib.filterAttrs (_: lib.isOption) opts));
    in
    lib.concatMapStringsSep "\n\n" (name: fmtOption name opts.${name}) names;

  section = title: body: "${bold (green title)}\n\n${body}";
in
let
  azurite = section "services.azurite" (fmtSection ../azurite "azurite");
  blob = section "services.azurite-blob" (fmtSection ../azurite-blob "azurite-blob");
  table = section "services.azurite-table" (fmtSection ../azurite-table "azurite-table");
  queue = section "services.azurite-queue" (fmtSection ../azurite-queue "azurite-queue");
in
{
  azurite = azurite + "\n";
  blob = blob + "\n";
  table = table + "\n";
  queue = queue + "\n";
}
