#!/usr/bin/env bash
# Terminal welcome: the welcome banner (uwelcome) then the system information
# (fastfetch), like a fresh Bluefin terminal with fastfetch added. Replaces
# projectbluefin/common's uwelcome.sh, which only shows the banner.
# `uwelcome toggle` turns both off, or back on.

if [ -e ~/.config/no-show-user-motd ]; then
	mkdir -p ~/.config/uwelcome
	mv ~/.config/no-show-user-motd ~/.config/uwelcome/disabled 2>/dev/null
fi

# Interactive terminals only (not scripts, `bash -l -c`, ssh commands), never
# for root: root shells (sudo su -, sudo -i) have no desktop session, and the
# portal lookup uwelcome makes can stall the prompt when the portal cannot
# start. Guard against double greetings when a shell chains into another
# (bash login shell starting fish): the first one wins.
case $- in
*i*)
	if [ -t 1 ] && [ "$(id -u)" != "0" ] && [ -z "${UWELCOME_SHOWN-}" ]; then
		UWELCOME_SHOWN=1
		export UWELCOME_SHOWN
		uwelcome
		if [ ! -e "${XDG_CONFIG_HOME:-$HOME/.config}/uwelcome/disabled" ]; then
			ublue-fastfetch
		fi
	fi
	;;
esac
