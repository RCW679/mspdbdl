use std::{env, io, str, fs, u16};
pub struct Version(u16, u16, u16, u16);
impl Version {
    pub fn convert(self: &Self) -> u64 {
        return (self.0 as u64) << 48 | (self.1 as u64) << 32 |
               (self.2 as u64) << 16 | (self.3 as u64);
    }
    pub fn parse(s: &str) -> Option<Version> {
        let split = Vec::from_iter(s.split('.'));
        if split.len() != 4 {
            return Option::None;
        }
        let mut ver = Version(0, 0, 0, 0);
        ver.0 = split[0].parse::<u16>().ok()?;
        ver.1 = split[1].parse::<u16>().ok()?;
        ver.2 = split[2].parse::<u16>().ok()?;
        ver.3 = split[3].parse::<u16>().ok()?;
        return Option::Some(ver);
    }
}
pub fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let ct = str::from_utf8(fs::read("Cargo.toml")?.as_slice()).unwrap().parse::<toml::Table>().unwrap();
        let ver = Version::parse(ct["package"]["metadata"]["winresource"]["ProductVersion"].as_str().unwrap()).unwrap();
        winresource::WindowsResource::new()
            // This path can be absolute, or relative to your crate root.
            .set_icon("src/icon.ico")
            .set_version_info(winresource::VersionInfo::FILEVERSION, ver.convert())
            .set_version_info(winresource::VersionInfo::PRODUCTVERSION, ver.convert())
            .compile()?;
    }
    Ok(())
}