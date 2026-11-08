//! The per-base interface: what differs between Arch and Fedora live images, behind one trait, so the disk, copy and
//! system steps stay shared. The base comes from os-release's `ID_LIKE`, so no flag picks it.

use std::path::{Path, PathBuf};

use crate::disk::SUBVOLUMES;
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
    /// What puts the kernel and its initramfs on the ESP, mounted at the target's /boot: files to write (path, text),
    /// then commands. `cmdline` is the kernel command line (`boot::cmdline`). Limine's program is on the ESP by then.
    fn initramfs(&self, target: &Path, cmdline: &str) -> (Vec<(PathBuf, String)>, Vec<Cmd>);
    /// The kernel's and the initramfs' file names at the ESP's root, for the one entry the installer writes into
    /// `limine.conf`; `None` when the base's own kernel tools wrote `limine.conf` in `initramfs`.
    fn kernel_files(&self) -> Option<(&'static str, &'static str)>;
    /// Commands that give `path` under the target (all of it when empty) its policy's labels (Fedora: SELinux), after
    /// the account files change and after every write, before the firmware entry.
    fn relabel(&self, _target: &Path, _path: &Path) -> Vec<Cmd> {
        Vec::new()
    }
}

/// mkinitcpio's drop-in in the target, read after /etc/mkinitcpio.conf.
const ARCH_HOOKS: &str = "etc/mkinitcpio.conf.d/stackific.conf";
/// Its text: mkinitcpio 42's default HOOKS with `sd-encrypt` after `block` and `stackific-boot-failure` last.
const ARCH_HOOKS_CONF: &str = "# Written by stackific-installer: Arch's default hooks, plus sd-encrypt, which unlocks the \
encrypted root, and stackific-boot-failure, which shows a plain screen when the start fails\n\
HOOKS=(base systemd autodetect microcode modconf kms keyboard sd-vconsole block sd-encrypt filesystems fsck \
stackific-boot-failure)\n";

/// Whether the installer runs on aarch64, where each base boots its kernel differently (see `Arch` and Fedora's
/// stackific-boot plugin).
const AARCH64: bool = cfg!(target_arch = "aarch64");

/// The live image's pacman hook that copies Arch Linux ARM's /boot/Image to /boot/vmlinuz-linux for mkarchiso
/// (arch/iso/airootfs): live-only glue, so installed systems keep no second copy of the kernel.
const ARCH_LIVE_HOOK: &str = "etc/pacman.d/hooks/stackific-vmlinuz.hook";

/// Arch Linux (and Arch Linux ARM): archiso's live image.
pub struct Arch;

/// Arch's kernel step for x86_64 or, with `aarch64`, Arch Linux ARM. x86_64: mkinitcpio's own pacman-hook script,
/// with every kernel on stdin as pacman runs it, copies the kernel to /boot/vmlinuz-<pkgbase> and builds
/// /boot/initramfs-<pkgbase>.img. Arch Linux ARM's linux-aarch64 has no /usr/lib/modules/*/vmlinuz for that script:
/// the package itself puts the kernel at /boot/Image, which mkarchiso empties from the live root, so the kernel comes
/// from the boot medium (`medium`; archiso's file is the live hook's copy of /boot/Image), and `mkinitcpio -P` builds
/// /boot/initramfs-linux.img from linux-aarch64's preset.
fn arch_kernel(target: &Path, medium: &Path, aarch64: bool) -> Vec<Cmd> {
    let chroot = |script: &str| {
        Cmd::new("arch-chroot").args([&*target.to_string_lossy(), "/usr/bin/bash", "-c", script])
    };
    if !aarch64 {
        return vec![chroot(
            "printf '%s\\n' usr/lib/modules/*/vmlinuz | /usr/share/libalpm/scripts/mkinitcpio install",
        )];
    }
    vec![
        Cmd::new("install").args([
            "-D",
            "-m",
            "0644",
            &*medium
                .join("arch/boot/aarch64/vmlinuz-linux")
                .to_string_lossy(),
            &*target.join("boot/Image").to_string_lossy(),
        ]),
        chroot("mkinitcpio -P"),
    ]
}

/// The kernel and initramfs file names linux (x86_64) or linux-aarch64 (with `aarch64`) and their mkinitcpio presets
/// keep current at the ESP's root.
fn arch_kernel_files(aarch64: bool) -> (&'static str, &'static str) {
    if aarch64 {
        ("Image", "initramfs-linux.img")
    } else {
        ("vmlinuz-linux", "initramfs-linux.img")
    }
}

impl Base for Arch {
    /// archiso's hook mounts the squashfs root here, the lower layer of the live overlay.
    fn live_root(&self) -> &'static Path {
        Path::new("/run/archiso/airootfs")
    }

    fn live_medium(&self) -> &'static Path {
        Path::new("/run/archiso/bootmnt")
    }

    /// archiso's mkinitcpio drop-in, the live image's kernel-copy hook, and the installer package with the
    /// dependencies only it needed (`-Rns`), so the installed system has no installer and no launcher.
    fn live_state(&self, target: &Path) -> (Vec<PathBuf>, Vec<Cmd>) {
        let files = vec![
            target.join("etc/mkinitcpio.conf.d/archiso.conf"),
            target.join(ARCH_LIVE_HOOK),
        ];
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

    /// The kernel on the ESP and its initramfs from the preset (Arch's default: no fallback image), in `arch-chroot`,
    /// which gives mkinitcpio /dev, /proc and /sys for the autodetect hook (`arch_kernel`).
    /// First the hooks (BT-04): Arch's defaults plus `sd-encrypt`, which unlocks the root, and stackific-boot's
    /// `stackific-boot-failure`, the plain screen after a failed start. The installer writes the drop-in, not a package:
    /// a package's would sort after archiso's and change the live image's initramfs. `limine.conf` carries the command
    /// line.
    fn initramfs(&self, target: &Path, _cmdline: &str) -> (Vec<(PathBuf, String)>, Vec<Cmd>) {
        let files = vec![(target.join(ARCH_HOOKS), ARCH_HOOKS_CONF.to_string())];
        (files, arch_kernel(target, self.live_medium(), AARCH64))
    }

    fn kernel_files(&self) -> Option<(&'static str, &'static str)> {
        Some(arch_kernel_files(AARCH64))
    }
}

/// Fedora: a kiwi live image booted by dracut's dmsquash-live, with kernel-install and dracut for the kernel.
pub struct Fedora;

/// The live image's GRUB on x86_64 or, with `aarch64`, on aarch64 (fedora/iso/config.kiwi), which installed systems do
/// not use (Limine starts them); removing it also takes GRUB's kernel-install plugins, which would otherwise write BLS
/// entries and call grub2-editenv on every kernel update.
fn live_grub(aarch64: bool) -> [&'static str; 3] {
    if aarch64 {
        [
            "grub2-efi-aa64",
            "grub2-efi-aa64-cdboot",
            "grub2-efi-aa64-modules",
        ]
    } else {
        [
            "grub2-efi-x64",
            "grub2-efi-x64-cdboot",
            "grub2-efi-x64-modules",
        ]
    }
}

impl Base for Fedora {
    /// dmsquash-live bind-mounts the root image here, the lower layer of the live overlay.
    fn live_root(&self) -> &'static Path {
        Path::new("/run/rootfsbase")
    }

    fn live_medium(&self) -> &'static Path {
        Path::new("/run/initramfs/live")
    }

    /// The kernel interfaces the chroot below and rpm's scriptlets need, mounted as arch-chroot mounts them (`/run`
    /// not recursively, so the target's own mount under /run stays out), then the installer with the dependencies
    /// only it needed, and the live image's GRUB, removed offline by dnf5. The installer's final `umount --recursive`
    /// takes the mounts down.
    fn live_state(&self, target: &Path) -> (Vec<PathBuf>, Vec<Cmd>) {
        let t = |p: &str| target.join(p).to_string_lossy().to_string();
        let cmds = vec![
            Cmd::new("mount").args(["-t", "proc", "proc", &t("proc")]),
            Cmd::new("mount").args(["-t", "sysfs", "sys", &t("sys")]),
            Cmd::new("mount").args(["--bind", "/dev", &t("dev")]),
            Cmd::new("mount").args(["--bind", "/run", &t("run")]),
            // GRUB's packages mark themselves protected (/etc/dnf/protected.d/); this removal alone lifts that
            Cmd::new("dnf5")
                .args([format!("--installroot={}", target.display())])
                .args([
                    "--setopt=protected_packages=",
                    "-y",
                    "remove",
                    "stackific-installer",
                ])
                .args(live_grub(AARCH64)),
        ];
        (Vec::new(), cmds)
    }

    /// The command line for kernel-install's plugins, then kernel-install in the target for every kernel, as the
    /// kernel package's own scriptlet runs it: Fedora's dracut plugin builds /boot/initramfs-<version>.img and
    /// stackific-boot's plugin copies the kernel and writes `limine.conf`.
    fn initramfs(&self, target: &Path, cmdline: &str) -> (Vec<(PathBuf, String)>, Vec<Cmd>) {
        let files = vec![(target.join("etc/kernel/cmdline"), format!("{cmdline}\n"))];
        let cmd = Cmd::new("chroot").args([
            &*target.to_string_lossy(),
            "/usr/bin/bash",
            "-c",
            r#"for k in /usr/lib/modules/*/vmlinuz; do v=${k%/vmlinuz}; kernel-install add "${v##*/}" "$k"; done"#,
        ]);
        (files, vec![cmd])
    }

    fn kernel_files(&self) -> Option<(&'static str, &'static str)> {
        None
    }

    /// The policy's labels for what the installer wrote under `path` (the user's home among them), from the target's
    /// own file contexts (setfiles also reads their `.homedirs` and `.local` files). setfiles does not cross into other
    /// mounts, so all of the target means `/` and every subvolume's mount point. The ESP holds no labels, and the
    /// kernel interfaces mounted above are not the target's files.
    fn relabel(&self, target: &Path, path: &Path) -> Vec<Cmd> {
        let t = |p: &str| {
            let full = target.join(p.trim_start_matches('/'));
            full.to_string_lossy().trim_end_matches('/').to_string()
        };
        let mut cmd = Cmd::new("setfiles").args(["-r", &*target.to_string_lossy()]);
        for skip in ["boot", "proc", "sys", "dev", "run"] {
            cmd = cmd.args(["-e", &t(skip)]);
        }
        cmd = cmd.args([t("etc/selinux/targeted/contexts/files/file_contexts")]);
        vec![if path.as_os_str().is_empty() {
            cmd.args(SUBVOLUMES.iter().map(|(_, m)| t(m)))
        } else {
            cmd.args([t(&path.to_string_lossy())])
        }]
    }
}

/// The base of the running system, from `ID_LIKE`.
pub fn detect(os: &OsRelease) -> Option<Box<dyn Base>> {
    os.id_like.split_whitespace().find_map(|l| match l {
        "arch" => Some(Box::new(Arch) as Box<dyn Base>),
        "fedora" => Some(Box::new(Fedora) as Box<dyn Base>),
        _ => None,
    })
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
            [
                PathBuf::from("/t/etc/mkinitcpio.conf.d/archiso.conf"),
                PathBuf::from("/t/etc/pacman.d/hooks/stackific-vmlinuz.hook")
            ]
        );
        assert_eq!(
            cmds[0].line(),
            "pacman --sysroot=/t --remove --nosave --recursive --noconfirm stackific-installer"
        );
        let (files, cmds) = base.initramfs(Path::new("/t"), "root=UUID=x");
        assert_eq!(
            files[0].0,
            PathBuf::from("/t/etc/mkinitcpio.conf.d/stackific.conf")
        );
        assert!(files[0].1.ends_with(
            "\nHOOKS=(base systemd autodetect microcode modconf kms keyboard sd-vconsole block sd-encrypt filesystems fsck \
             stackific-boot-failure)\n"
        ));
        let lines = |c: Vec<Cmd>| c.iter().map(Cmd::line).collect::<Vec<_>>();
        assert_eq!(
            lines(cmds),
            lines(arch_kernel(Path::new("/t"), base.live_medium(), AARCH64))
        );
        assert_eq!(base.kernel_files(), Some(arch_kernel_files(AARCH64)));
        let x86 = lines(arch_kernel(Path::new("/t"), base.live_medium(), false));
        assert_eq!(
            x86,
            [
                "arch-chroot /t /usr/bin/bash -c \"printf '%s\\\\n' usr/lib/modules/*/vmlinuz | /usr/share/libalpm/scripts/mkinitcpio install\""
            ]
        );
        assert_eq!(
            arch_kernel_files(false),
            ("vmlinuz-linux", "initramfs-linux.img")
        );
        // Arch Linux ARM: the kernel from the boot medium to the path linux-aarch64 owns, then its preset
        let arm = lines(arch_kernel(Path::new("/t"), base.live_medium(), true));
        assert_eq!(
            arm,
            [
                "install -D -m 0644 /run/archiso/bootmnt/arch/boot/aarch64/vmlinuz-linux /t/boot/Image",
                "arch-chroot /t /usr/bin/bash -c \"mkinitcpio -P\""
            ]
        );
        assert_eq!(arch_kernel_files(true), ("Image", "initramfs-linux.img"));
        assert!(base.relabel(Path::new("/t"), Path::new("")).is_empty());
        assert!(detect(&OsRelease::parse("ID=stackific\nID_LIKE=debian\n")).is_none());
    }

    #[test]
    fn fedora_from_id_like() {
        let os = OsRelease::parse("ID=stackific\nID_LIKE=fedora\n");
        let base = detect(&os).unwrap();
        assert_eq!(base.live_root(), Path::new("/run/rootfsbase"));
        assert_eq!(base.live_medium(), Path::new("/run/initramfs/live"));
        let (files, cmds) = base.live_state(Path::new("/t"));
        assert!(files.is_empty());
        let lines: Vec<_> = cmds.iter().map(Cmd::line).collect();
        assert_eq!(
            lines,
            [
                "mount -t proc proc /t/proc".to_string(),
                "mount -t sysfs sys /t/sys".to_string(),
                "mount --bind /dev /t/dev".to_string(),
                "mount --bind /run /t/run".to_string(),
                format!(
                    "dnf5 --installroot=/t --setopt=protected_packages= -y remove stackific-installer {}",
                    live_grub(AARCH64).join(" ")
                )
            ]
        );
        assert_eq!(
            live_grub(false),
            [
                "grub2-efi-x64",
                "grub2-efi-x64-cdboot",
                "grub2-efi-x64-modules"
            ]
        );
        assert_eq!(
            live_grub(true),
            [
                "grub2-efi-aa64",
                "grub2-efi-aa64-cdboot",
                "grub2-efi-aa64-modules"
            ]
        );
        let (files, cmds) = base.initramfs(Path::new("/t"), "root=UUID=x rootflags=subvol=/@ rw");
        assert_eq!(
            files,
            [(
                PathBuf::from("/t/etc/kernel/cmdline"),
                "root=UUID=x rootflags=subvol=/@ rw\n".to_string()
            )]
        );
        assert_eq!(
            cmds[0].line(),
            r#"chroot /t /usr/bin/bash -c "for k in /usr/lib/modules/*/vmlinuz; do v=${k%/vmlinuz}; kernel-install add \"${v##*/}\" \"$k\"; done""#
        );
        assert_eq!(base.kernel_files(), None);
        assert_eq!(
            base.relabel(Path::new("/t"), Path::new(""))[0].line(),
            "setfiles -r /t -e /t/boot -e /t/proc -e /t/sys -e /t/dev -e /t/run /t/etc/selinux/targeted/contexts/files/file_contexts \
             /t /t/home /t/root /t/srv /t/var/cache /t/var/tmp /t/var/log /t/swap /t/var/lib/flatpak /t/var/lib/libvirt/images \
             /t/var/lib/containers /t/var/lib/docker"
        );
        assert!(
            base.relabel(Path::new("/t"), Path::new("etc"))[0]
                .line()
                .ends_with("file_contexts /t/etc")
        );
    }
}
