const fn get_conda_architecture() -> Option<&'static str> {
    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "x86_64")]
    return Some("linux-64");

    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "aarch64")]
    return Some("linux-aarch64");

    #[cfg(target_os = "linux")]
    #[cfg(target_arch = "powerpc64le")]
    return Some("linux-ppc64le");

    #[cfg(target_os = "macos")]
    #[cfg(target_arch = "x86_64")]
    return Some("osx-64");

    #[cfg(target_os = "macos")]
    #[cfg(target_arch = "aarch64")]
    return Some("osx-arm64");

    #[allow(unreachable_code)]
    None
}

pub(crate) fn get_mamba_url() -> Option<String> {
    let arch = get_conda_architecture()?;
    let url = format!("https://micro.mamba.pm/api/micromamba/{arch}/latest");
    Some(url)
}
