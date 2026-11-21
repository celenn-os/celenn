#!/usr/bin/bash
# Parental controls (the Parental controls app, built with the tour in
# Containerfile.helpers; see build_files/first_steps/src/parental*.rs):
# what its service and its login check need from the system.
set -euox pipefail

# dnsmasq: the family web filter (cosmic-blue-family-dns.service); nftables:
# sends the child accounts' DNS queries to it; acl (setfacl): closes
# systemd-resolved's socket to them; libnotify (notify-send): the warnings
# before the session closes
dnf5 -y install acl dnsmasq libnotify nftables

# The login check: during bedtime, or once the day's screen time is used up,
# the login screen refuses a child account with a message saying when it can
# log in again (pam_exec prints it). Accounts without rules are not affected.
# The login screen (cosmic-greeter) logs in through greetd.
pam=/etc/pam.d/greetd
test -f "${pam}"
line='account    required     pam_exec.so quiet stdout /usr/libexec/cosmic-blue-parental-daemon login-check'
if ! grep -qF 'cosmic-blue-parental-daemon login-check' "${pam}"; then
    # Before the first account line, so that it runs whatever follows
    awk -v line="${line}" '!done && /^account/ { print line; done = 1 } { print }' "${pam}" > "${pam}.new"
    grep -qF 'cosmic-blue-parental-daemon login-check' "${pam}.new"
    cat "${pam}.new" > "${pam}"
    rm -f "${pam}.new"
fi
cat "${pam}"
