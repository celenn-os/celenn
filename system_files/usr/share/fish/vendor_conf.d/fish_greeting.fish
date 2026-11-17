# Terminal welcome for fish: the welcome banner (uwelcome) then the system
# information (fastfetch), as /etc/profile.d/uwelcome.sh does for bash.
# Replaces projectbluefin/common's fish_greeting, which only shows the banner.
function fish_greeting

    if test -e ~/.config/no-show-user-motd
        mkdir -p ~/.config/uwelcome
        mv ~/.config/no-show-user-motd ~/.config/uwelcome/disabled 2>/dev/null
    end

    # Skip for root (no desktop session, portal lookups can stall), and skip
    # when a parent shell already greeted (bash login shell starting fish).
    if test (id -u) = 0
        return
    end
    if set -q UWELCOME_SHOWN
        return
    end
    set -gx UWELCOME_SHOWN 1

    uwelcome
    set -l config_home $HOME/.config
    if set -q XDG_CONFIG_HOME
        set config_home $XDG_CONFIG_HOME
    end
    if not test -e $config_home/uwelcome/disabled
        ublue-fastfetch
    end
end
