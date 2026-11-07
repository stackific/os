//! The per-base interface: what differs between Arch and Fedora live images, behind one trait, so the disk, copy and
//! system steps stay shared. The base comes from os-release's `ID_LIKE`, so no flag picks it.

use std::path::{Path, PathBuf};

use crate::os_release::OsRelease;
use crate::run::Cmd;

/// One base's live image and its own clean-up.
pub trait Base: Send + Sync {
    /// The live image's pristine root, read-only: the copy's source.
    fn live_root(&self) -> &'static Path;
    /// Where the boot medium is mounted: the drive under it is never offered.
    fn live_medium(&self) -> &'static Path;
    /// The base's own live state in the copied root at `target`: files to remove, then commands to run.
    fn live_state(&self, target: &Path) -> (Vec<PathBuf>, Vec<Cmd>);
    /// The commands that put the kernel and its initramfs on the ESP, mounted at the target's /boot.
    fn initramfs(&self, target: &Path) -> Vec<Cmd>;
    /// The kernel's and the initramfs' file names at the ESP's root, which Limine's entry names.
    fn kernel_files(&self) -> (&'static str, &'static str);
}

/// Arch Linux (and Arch Linux ARM): archiso's live image.
pub struct Arch;

impl Base for Arch {
    /// archiso's hook mounts the squashfs root here, the lower layer of the live overlay.
    fn live_root(&self) -> &'static Path {
        Path::new("/run/archiso/airootfs")
    }

    fn live_medium(&self) -> &'static Path {
        Path::new("/run/archiso/bootmnt")
    }

    /// archiso's mkinitcpio drop-in, and the installer package with the dependencies only it needed (`-Rns`), so the
    /// installed system has no installer and no launcher.
    fn live_state(&self, target: &Path) -> (Vec<PathBuf>, Vec<Cmd>) {
        let files = vec![target.join("etc/mkinitcpio.conf.d/archiso.conf")];
        let pacman = Cmd::new("pacman")
            .args([format!("--sysroot={}", target.display())])
            .args([
                "--remove",
                "--nosave",
                "--recursive",
                "--noconfirm",
                "stackific-installer",
            ]);
        (files, vec![pacman])
    }

    /// mkinitcpio's own pacman-hook script, run in the target with every kernel on stdin, as pacman runs it when it
    /// installs a kernel: it copies the kernel to /boot/vmlinuz-<pkgbase> and builds /boot/initramfs-<pkgbase>.img
    /// from the preset (Arch's default: no fallback image) with Arch's default hooks. arch-chroot gives it /dev, /proc
    /// and /sys, which the autodetect hook reads.
    fn initramfs(&self, target: &Path) -> Vec<Cmd> {
        vec![Cmd::new("arch-chroot").args([
            &*target.to_string_lossy(),
            "/usr/bin/bash",
            "-c",
            "printf '%s\\n' usr/lib/modules/*/vmlinuz | /usr/share/libalpm/scripts/mkinitcpio install",
        ])]
    }

    /// The paths mkinitcpio's preset writes for the `linux` kernel; its pacman hooks keep them current on updates.
    fn kernel_files(&self) -> (&'static str, &'static str) {
        ("vmlinuz-linux", "initramfs-linux.img")
    }
}

/// The base of the running system, from `ID_LIKE`.
pub fn detect(os: &OsRelease) -> Option<Box<dyn Base>> {
    os.id_like
        .split_whitespace()
        .any(|l| l == "arch")
        .then(|| Box::new(Arch) as Box<dyn Base>)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arch_from_id_like() {
        let os = OsRelease::parse("ID=stackific\nID_LIKE=arch\n");
        let base = detect(&os).unwrap();
        assert_eq!(base.live_root(), Path::new("/run/archiso/airootfs"));
        let (files, cmds) = base.live_state(Path::new("/t"));
        assert_eq!(
            files,
            [PathBuf::from("/t/etc/mkinitcpio.conf.d/archiso.conf")]
        );
        assert_eq!(
            cmds[0].line(),
            "pacman --sysroot=/t --remove --nosave --recursive --noconfirm stackific-installer"
        );
        assert_eq!(
            base.initramfs(Path::new("/t"))[0].line(),
            "arch-chroot /t /usr/bin/bash -c \"printf '%s\\\\n' usr/lib/modules/*/vmlinuz | /usr/share/libalpm/scripts/mkinitcpio install\""
        );
        assert_eq!(
            base.kernel_files(),
            ("vmlinuz-linux", "initramfs-linux.img")
        );
        assert!(detect(&OsRelease::parse("ID=stackific\nID_LIKE=fedora\n")).is_none());
    }
}
