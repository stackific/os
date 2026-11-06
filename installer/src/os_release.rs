//! The installer's names, labels, default computer name and accent come from `/etc/os-release` (os-release(5)), which
//! stackific-release renders from the branding file, so nothing here hard-codes the distribution.

/// The os-release fields the installer uses.
#[derive(Clone, Debug, PartialEq)]
pub struct OsRelease {
    /// `NAME`, such as "Stackific OS": titles, partition names and labels.
    pub name: String,
    /// `ID`, such as "stackific": the btrfs label and, in capitals, the FAT label.
    pub id: String,
    /// `ID_LIKE`, which picks the base (`arch`).
    pub id_like: String,
    /// `DEFAULT_HOSTNAME`, the computer name the You screen starts with (the ID when unset).
    pub default_hostname: String,
    /// `ANSI_COLOR` as 24-bit RGB, when it is one (`38;2;R;G;B`): the accent.
    pub accent: Option<(u8, u8, u8)>,
}

impl OsRelease {
    /// Parses os-release text: `KEY=value` lines, values optionally quoted.
    pub fn parse(text: &str) -> Self {
        let get = |key: &str| {
            text.lines()
                .filter_map(|l| l.trim().split_once('='))
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.trim().trim_matches(['"', '\'']).to_string())
                .unwrap_or_default()
        };
        let id = get("ID");
        let default_hostname = Some(get("DEFAULT_HOSTNAME"))
            .filter(|h| !h.is_empty())
            .unwrap_or_else(|| id.clone());
        let ansi = get("ANSI_COLOR");
        let rgb: Vec<u8> = ansi.split(';').filter_map(|p| p.parse().ok()).collect();
        let accent = match (ansi.starts_with("38;2;"), rgb.as_slice()) {
            (true, [_, _, r, g, b]) => Some((*r, *g, *b)),
            _ => None,
        };
        Self {
            name: get("NAME"),
            id,
            id_like: get("ID_LIKE"),
            default_hostname,
            accent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_rendered_file() {
        let o = OsRelease::parse(
            "NAME=\"Stackific OS\"\nID=stackific\nDEFAULT_HOSTNAME=stackific\nANSI_COLOR=\"38;2;0;122;85\"\nID_LIKE=arch\n",
        );
        assert_eq!(o.name, "Stackific OS");
        assert_eq!(
            (
                o.id.as_str(),
                o.id_like.as_str(),
                o.default_hostname.as_str()
            ),
            ("stackific", "arch", "stackific")
        );
        assert_eq!(o.accent, Some((0, 122, 85)));
        assert_eq!(OsRelease::parse("ID=x\nANSI_COLOR=\"0;32\"").accent, None);
        assert_eq!(OsRelease::parse("ID=x").default_hostname, "x");
    }
}
