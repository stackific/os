# Shared lefthook settings. Import this from any module that runs lefthook.

# Pinned lefthook (github.com/evilmartians/lefthook) and the SHA-256 of its release asset per host
lefthook_version := "2.2.2"
lefthook_asset := "lefthook_" + lefthook_version + "_" + (if os() == "macos" { "MacOS" } else { "Linux" }) + "_" + (if arch() == "aarch64" { "arm64" } else { "x86_64" })
lefthook_sha256 := if lefthook_asset =~ "MacOS_arm64$" { "049c648578c056a180519c4d8c31d5436dbb9df74369b2e29611aae99d7e9b36" } else if lefthook_asset =~ "Linux_arm64$" { "a4d4bf78d902af1a148bf0662af08e13b791061e46e6e2183b0660eed96b43f6" } else { "4fbf89eb7748c6aed84d833ee1a29edac48fae614468ca9a82896ec32f2800b6" }

lefthook := justfile_directory() / ".tools" / "lefthook" / "lefthook"
