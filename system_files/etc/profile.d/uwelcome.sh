# shellcheck shell=sh
# Terminal welcome banner (uwelcome). Replaces the profile.d/uwelcome.sh of
# projectbluefin/common, which greets in every shell that sources
# /etc/profile and exports UWELCOME_SHOWN so that a chained shell does not
# greet twice. The COSMIC session itself is started through /etc/profile
# (greetd sources it), so that export reached the whole desktop and every
# terminal opened from it showed no banner. Here only interactive shells on
# a terminal greet and set the guard.

if [ -e ~/.config/no-show-user-motd ]; then
	mkdir -p ~/.config/uwelcome
	mv ~/.config/no-show-user-motd ~/.config/uwelcome/disabled 2>/dev/null
fi

# Skip for root: root shells (sudo su -, sudo -i) have no desktop session,
# and the portal lookup uwelcome makes can stall the prompt when the portal
# cannot start.
case $- in
*i*)
	if [ -t 1 ] && [ "$(id -u)" != "0" ] && [ -z "${UWELCOME_SHOWN-}" ]; then
		UWELCOME_SHOWN=1
		export UWELCOME_SHOWN
		uwelcome
	fi
	;;
esac
