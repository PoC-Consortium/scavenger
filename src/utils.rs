cfg_if! {
    if #[cfg(unix)] {
        use std::process::Command;

        pub fn get_device_id(path: &String) -> String {
            let output = Command::new("stat")
                .arg(path)
                .arg("-c %D")
                .output()
                .expect("failed to execute");
            String::from_utf8(output.stdout).expect("not utf8")
        }

        pub fn get_sector_size(path: &String) -> u64 {
            let output = Command::new("df")
                .arg(path)
                .arg("--output=source")
                .output()
                .expect("failed to execute");
            let source = String::from_utf8(output.stdout).expect("not utf8");
            let source = source.split("\n").collect::<Vec<&str>>()[1];

            let output = Command::new("lsblk")
                .arg(source)
                .arg("-o")
                .arg("PHY-SeC")
                .output()
                .expect("failed to execute");

            let sector_size = String::from_utf8(output.stdout).expect("not utf8");
            let sector_size = sector_size.split("\n").collect::<Vec<&str>>()[1].trim();

            sector_size.parse::<u64>().unwrap()
        }
    } else {
        extern crate winapi;
        use std::os::windows::ffi::OsStrExt;
        use std::ffi::OsStr;
        use std::iter::once;
        use std::ffi::CString;
        use std::path::Path;

        pub fn get_device_id(path: &String) -> String {
            let path_encoded: Vec<u16> = OsStr::new(path).encode_wide().chain(once(0)).collect();
            let mut volume_encoded: Vec<u16> = OsStr::new(path).encode_wide().chain(once(0)).collect();
            if unsafe {
                winapi::um::fileapi::GetVolumePathNameW(
                    path_encoded.as_ptr(),
                    volume_encoded.as_mut_ptr(),
                    path.chars().count() as u32
                )
            } == 0  {
                panic!("get volume path name");
            };
            let res = String::from_utf16_lossy(&volume_encoded);
            let v: Vec<&str> = res.split('\u{00}').collect();
            String::from(v[0])
        }

        pub fn get_sector_size(path: &String) -> u64 {
            let path_encoded = Path::new(path);
            let parent_path = path_encoded.parent().unwrap().to_str().unwrap();
            let parent_path_encoded = CString::new(parent_path).unwrap();
            let mut sectors_per_cluster  = 0u32;
            let mut bytes_per_sector  = 0u32;
            let mut number_of_free_cluster  = 0u32;
            let mut total_number_of_cluster  = 0u32;
            if unsafe {
                winapi::um::fileapi::GetDiskFreeSpaceA(
                    parent_path_encoded.as_ptr(),
                    &mut sectors_per_cluster,
                    &mut bytes_per_sector,
                    &mut number_of_free_cluster,
                    &mut total_number_of_cluster
                )
            } == 0  {
                panic!("get sector size, filename={}",path);
            };
            bytes_per_sector as u64
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_device_id() {
        if cfg!(unix) {
            assert_ne!("", get_device_id(&"Cargo.toml".to_string()));
        }
    }
}
