pub const APPLE: [&str; 4] = ["macos", "darwin", "mac", "dmg"];
pub const LINUX: [&str; 1] = ["linux"];
pub const AMD64: [&str; 4] = ["x64", "x86_64", "amd64", "64bit"];
pub const ARM64: [&str; 2] = ["aarch64", "arm64"];

#[derive(Debug)]
pub struct System {
    arch: Arch,
    os: OS,
}

#[derive(Clone, Copy, Debug)]
pub enum OS {
    Windows,
    Linux,
    Darwin,
    Unknown,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Arch {
    amd64,
    aarch64,
    Unknown,
}

impl System {
    pub fn new() -> Self {
        let os = if cfg!(target_os = "linux") {
            OS::Linux
        } else if cfg!(target_os = "macos") {
            OS::Darwin
        } else if cfg!(windows) {
            OS::Windows
        } else {
            panic!("This platform dose not support yet!");
        };
        let arch = if cfg!(target_arch = "x86_64") {
            Arch::amd64
        } else if cfg!(target_arch = "aarch64") {
            Arch::aarch64
        } else {
            panic!("This arch dose not support yet!");
        };
        Self { os, arch }
    }

    pub fn os(&self) -> OS {
        self.os
    }

    pub fn arch(&self) -> Arch {
        self.arch
    }
}

// pub struct SysDetector {}
// impl SysDetector {
//     fn detect(name: &str) -> System {
//         for
//     }
// }
