//! Provides functions for determining the paths of files and directories
//! in a standard Volta layout in Unix-based operating systems.

use std::env;
use std::io;
use std::os::unix;
use std::path::{Path, PathBuf};

use dirs;

use crate::error::ErrorDetails;
use volta_fail::Fallible;

use super::{node_archive_root_dir_name, node_image_dir, shim_dir, volta_home};

// These are taken from: https://nodejs.org/dist/index.json and are used
// by `path::archive_root_dir` to determine the root directory of the
// contents of a Node installer archive.

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        /// The OS component of a Node distribution tarball's name.
        pub const OS: &'static str = "darwin";
    } else if #[cfg(target_os = "linux")] {
        /// The OS component of a Node distribution tarball's name.
        pub const OS: &'static str = "linux";
    } else {
        compile_error!("Unsupported target_os variant of unix (expected 'macos' or 'linux').");
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86")] {
        /// The system architecture component of a Node distribution tarball's name.
        pub const ARCH: &'static str = "x86";
    } else if #[cfg(target_arch = "x86_64")] {
        /// The system architecture component of a Node distribution tarball's name.
        pub const ARCH: &'static str = "x64";
    } else {
        compile_error!("Unsupported target_arch variant of unix (expected 'x86' or 'x64').");
    }
}

// ~/
//     .volta/
//         cache/                                          cache_dir
//             node/                                       node_cache_dir
//                 index.json                              node_index_file
//                 index.json.expires                      node_index_expiry_file
//         bin/                                            shim_dir
//             node                                        shim_file("node")
//             yarn
//             npm
//             npx
//             ...
//             ember
//         log/                                            log_dir
//         tools/                                          tools_dir
//             inventory/                                  inventory_dir
//                 node/                                   node_inventory_dir
//                     node-v4.8.4-linux-x64.tar.gz        node_distro_file_name("4.8.4")
//                     node-v4.8.4-npm                     node_npm_version_file("4.8.4")
//                     ...
//                 packages/                               package_inventory_dir
//                     ember-cli-3.7.1.tgz                 package_distro_file("ember-cli", "3.7.1")
//                     ember-cli-3.7.1.shasum              package_distro_shasum("ember-cli", "3.7.1")
//                 yarn/                                   yarn_inventory_dir
//             image/                                      image_dir
//                 node/                                   node_image_root_dir
//                     10.13.0/
//                         6.4.0/                          node_image_dir("10.13.0", "6.4.0")
//                             bin/                        node_image_bin_dir("10.13.0", "6.4.0")
//                 yarn/                                   yarn_image_root_dir
//                     1.7.0/                              yarn_image_dir("1.7.0")
//                 packages/                               package_image_root_dir
//                     ember-cli/
//                         3.7.1/                          package_image_dir("ember-cli", "3.7.1")
//             user/                                       user_toolchain_dir
//                 bins/
//                     tsc.json                            user_tool_bin_config("tsc")
//                 packages/                               user_package_dir
//                     ember-cli.json                      user_package_config_file("ember-cli")
//                 platform.json                           user_platform_file
//         volta                                           volta_file
//         shim                                            shim_executable
//         hooks.json                                      user_hooks_file

pub fn default_volta_home() -> Fallible<PathBuf> {
    let home = dirs::home_dir().ok_or(ErrorDetails::NoHomeEnvironmentVar)?;
    Ok(home.join(".volta"))
}

pub fn archive_extension() -> String {
    String::from("tar.gz")
}

pub fn node_image_bin_dir(node: &str, npm: &str) -> Fallible<PathBuf> {
    Ok(node_image_dir(node, npm)?.join("bin"))
}

pub fn node_archive_npm_package_json_path(version: &str) -> PathBuf {
    Path::new(&node_archive_root_dir_name(version))
        .join("lib")
        .join("node_modules")
        .join("npm")
        .join("package.json")
}

pub fn shim_file(toolname: &str) -> Fallible<PathBuf> {
    Ok(shim_dir()?.join(toolname))
}

// this is not currently used by anything
pub fn volta_file() -> Fallible<PathBuf> {
    Ok(volta_home()?.join("volta"))
}

// check that it exists - if not, check some other locations
pub fn shim_executable() -> Fallible<PathBuf> {
    // if VOLTA_SHIM is set, try that first
    // (not documented yet, as it's currently only used for testing)
    if let Some(shim_location) = env::var_os("VOLTA_SHIM") {
        return Ok(shim_location.into());
    }

    // default location for the shim executable
    // (this will be the case for the majority of installs)
    let default_shim_executable = volta_home()?.join("shim");
    if default_shim_executable.exists() {
        return Ok(default_shim_executable);
    }

    // when an RPM is installed as root, the shim will be here for non-root users
    // (this will be the case for some managed installs)
    let rpm_shim_executable = PathBuf::from("/usr/bin/volta-lib/shim");
    if rpm_shim_executable.exists() {
        return Ok(rpm_shim_executable);
    }

    Err(ErrorDetails::ShimExecutableNotFound.into())
}

pub fn env_paths() -> Fallible<Vec<PathBuf>> {
    Ok(vec![shim_dir()?])
}

/// Create a symlink. The `dst` path will be a symbolic link pointing to the `src` path.
pub fn create_file_symlink(src: PathBuf, dst: PathBuf) -> Result<(), io::Error> {
    unix::fs::symlink(src, dst)
}
