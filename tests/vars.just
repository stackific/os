# Pins for test assets the VM tests download (tests/vm/boot.sh's `asset`), imported by arch/justfile and fedora/justfile.

# cryptsetup's BitLocker test images: volumes Windows made, with their protectors listed in images.conf (checked
# 2026-11-07). The install tests write bitlk-aes-xts-128.img (passphrase and recovery key 64311DEA) to a partition.
bitlk_images_url := "https://gitlab.com/cryptsetup/cryptsetup/-/raw/v2.8.8/tests/bitlk-images.tar.xz"
bitlk_images_sha256 := "68bf5669f777668112d497234ebe2166b5feaf0e33b206b095b917f82937bd30"
