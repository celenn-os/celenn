# shellcheck shell=sh
# Replaces ublue-os/brew's brew.sh. Homebrew belongs to the first user
# (brew-setup.service gives /home/linuxbrew to UID 1000): other accounts,
# such as the login screen's, which runs through a login shell, cannot run it
# and printed "brew: Permission denied" on the screen before the login screen
# appeared. So brew is only set up when this user can run it.
# System programs come first in PATH, so brew cannot replace them (dbus...).
if [ -z "${HOMEBREW_PREFIX:-}" ] && [ -x /home/linuxbrew/.linuxbrew/bin/brew ]; then
  case ":${PATH}:" in
    *:/home/linuxbrew/.linuxbrew/bin:*) ;;
    *)
      eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv 2>/dev/null | grep -Ev '\bPATH=')"
      HOMEBREW_PREFIX="${HOMEBREW_PREFIX:-/home/linuxbrew/.linuxbrew}"
      export PATH="${PATH}:${HOMEBREW_PREFIX}/bin:${HOMEBREW_PREFIX}/sbin"
      ;;
  esac
fi
