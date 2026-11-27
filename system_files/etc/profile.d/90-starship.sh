# shellcheck shell=sh
command -v starship >/dev/null 2>&1 || return 0

if [ "$(basename "$(readlink /proc/$$/exe)")" = "bash" ]; then
  # Celenn OS prompt (powerline segments, Celenn Prompt font), unless the
  # user has their own ~/.config/starship.toml or STARSHIP_CONFIG, or is on
  # the Linux console, which cannot show its symbols. Checked again in each
  # shell, so a config created later is picked up by new terminals.
  CELENN_STARSHIP=/usr/share/celenn/starship.toml
  if [ "${STARSHIP_CONFIG:-}" = "${CELENN_STARSHIP}" ]; then
    unset STARSHIP_CONFIG
  fi
  if [ -z "${STARSHIP_CONFIG:-}" ] && [ ! -e "${HOME}/.config/starship.toml" ] &&
    [ "${TERM:-}" != linux ]; then
    export STARSHIP_CONFIG="${CELENN_STARSHIP}"
  fi
  unset CELENN_STARSHIP
  # The home directory is /var/home/<user> behind the /home symlink, and a new
  # terminal opens there: move to the same directory under $HOME, so the
  # prompt shows ~ instead of the full path
  case "${PWD}" in
    "/var${HOME}" | "/var${HOME}"/*) cd "${PWD#/var}" 2>/dev/null || true ;;
  esac
  eval "$(starship init bash)"
fi
