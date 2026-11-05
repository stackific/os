# shellcheck shell=bash
# Fedora x86_64 checks: boot.sh sources this file after the checks every image passes (`just fedora test`).

vm_until 'who' '^root +tty1 ' 30
vm_until 'getenforce' '^Enforcing$' 0
screenshot screen
