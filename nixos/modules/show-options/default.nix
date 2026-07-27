{ writeShellApplication
, writeText
, pkgs
}:
let
  options = import ./options-doc.nix { inherit pkgs; };
  mkFile = key: writeText "azurite-options-${key}" options.${key};
in
writeShellApplication {
  name = "azurite-show-options";
  text = ''
    sections=()
    seen_a=0 seen_b=0 seen_t=0 seen_q=0
    use_pager=1

    usage() {
      printf 'Usage: azurite-show-options [OPTIONS]\n'
      printf '\n'
      printf 'Show NixOS module options for Azurite services.\n'
      printf 'Defaults to showing services.azurite options.\n'
      printf '\n'
      printf 'Options:\n'
      printf '  -a, --azurite  Show services.azurite options\n'
      printf '  -b, --blob     Show services.azurite-blob options\n'
      printf '  -t, --table    Show services.azurite-table options\n'
      printf '  -q, --queue    Show services.azurite-queue options\n'
      printf '\n'
      printf '  --no-pager     Print directly without a pager\n'
      printf '  -h, --help     Show this help message\n'
    }

    add() {
      case "$1" in
      a) if [[ $seen_a -eq 0 ]]; then
        sections+=("${mkFile "azurite"}")
        seen_a=1
      fi ;;
      b) if [[ $seen_b -eq 0 ]]; then
        sections+=("${mkFile "blob"}")
        seen_b=1
      fi ;;
      t) if [[ $seen_t -eq 0 ]]; then
        sections+=("${mkFile "table"}")
        seen_t=1
      fi ;;
      q) if [[ $seen_q -eq 0 ]]; then
        sections+=("${mkFile "queue"}")
        seen_q=1
      fi ;;
      esac
    }

    while [[ $# -gt 0 ]]; do
      case "$1" in
      --azurite)
        add a
        shift
        ;;
      --blob)
        add b
        shift
        ;;
      --table)
        add t
        shift
        ;;
      --queue)
        add q
        shift
        ;;
      --no-pager)
        use_pager=0
        shift
        ;;
      --help)
        usage
        exit 0
        ;;
      -*)
        flags="''${1#-}"
        shift
        for ((i = 0; i < ''${#flags}; i++)); do
          flag="''${flags:$i:1}"
          case "$flag" in
          a | b | t | q) add "$flag" ;;
          h)
            usage
            exit 0
            ;;
          *)
            printf 'unknown flag: -%s\n\n' "$flag" >&2
            usage >&2
            exit 1
            ;;
          esac
        done
        ;;
      *)
        printf 'unknown argument: %s\n\n' "$1" >&2
        usage >&2
        exit 1
        ;;
      esac
    done

    if [[ ''${#sections[@]} -eq 0 ]]; then
      sections=("${mkFile "azurite"}")
    fi

    output() {
      local first=1
      for f in "''${sections[@]}"; do
        [[ $first -eq 0 ]] && printf '\n'
        cat "$f"
        first=0
      done
    }

    if [[ $use_pager -eq 1 ]] && [[ -t 1 ]]; then
      output | ''${PAGER:-less -R}
    else
      output
    fi
  '';
}
