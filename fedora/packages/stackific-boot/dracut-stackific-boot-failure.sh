#!/bin/bash
# dracut module 99stackific-boot-failure (stackific-boot): the plain screen after a failed start. Only in host-only
# images, the installed system's; the live image's initramfs, built without host-only, leaves it out.
# shellcheck disable=SC2154 # hostonly, systemdsystemconfdir and inst_simple come from dracut

check() {
  [[ $hostonly ]] && return 0
  return 255
}

depends() {
  echo systemd
}

install() {
  inst_simple /usr/lib/stackific/boot-failure /usr/lib/stackific/boot-failure
  inst_simple /usr/lib/stackific/boot-failure.conf "$systemdsystemconfdir/emergency.service.d/stackific-boot-failure.conf"
  inst_simple /usr/lib/stackific/boot-failure.conf "$systemdsystemconfdir/rescue.service.d/stackific-boot-failure.conf"
}
