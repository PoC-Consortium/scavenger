use std::error::Error;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const SCOOPS_IN_NONCE: u64 = 4096;
const SHABAL256_HASH_SIZE: u64 = 32;
pub const SCOOP_SIZE: u64 = SHABAL256_HASH_SIZE * 2;
const NONCE_SIZE: u64 = SCOOP_SIZE * SCOOPS_IN_NONCE;

// TODO: mining for multiple accounts
pub struct Plot {
    _account_id: u64,
    start_nonce: u64,
    nonces: u64,
    pub fh: File,
    read_offset: u64,
    use_direct_io: bool,
    pub name: String,
}

cfg_if! {
    if #[cfg(unix)] {
        use std::os::unix::fs::OpenOptionsExt;

        const O_DIRECT: i32 = 0o0040000;

        pub fn open_usining_direc_io<P: AsRef<Path>>(path: P) -> io::Result<File> {
            OpenOptions::new()
                .read(true)
                .custom_flags(O_DIRECT)
                .open(path)
        }
    } else {
        use std::os::windows::fs::OpenOptionsExt;

        const FILE_FLAG_NO_BUFFERING: u32 = 0x20000000;

        pub fn open_usining_direc_io<P: AsRef<Path>>(path: P) -> io::Result<File> {
            OpenOptions::new()
                .read(true)
                .custom_flags(FILE_FLAG_NO_BUFFERING)
                .open(path)
        }
    }
}

impl Plot {
    pub fn new(path: &PathBuf, use_direct_io: bool) -> Result<Plot, Box<Error>> {
        if !path.is_file() {
            return Err(From::from(format!(
                "{} is not a file",
                path.to_str().unwrap()
            )));
        }

        let plot_file = path.file_name().unwrap().to_str().unwrap();
        let parts: Vec<&str> = plot_file.split("_").collect();
        if parts.len() != 3 {
            return Err(From::from("plot file has wrong format"));
        }

        let account_id = parts[0].parse::<u64>()?;
        let start_nonce = parts[1].parse::<u64>()?;
        let nonces = parts[2].parse::<u64>()?;

        let size = fs::metadata(path)?.len();
        let exp_size = nonces * NONCE_SIZE;
        if size != exp_size as u64 {
            return Err(From::from(format!(
                "expected plot size {} but got {}",
                exp_size, size
            )));
        }

        let fh = if use_direct_io {
            open_usining_direc_io(path)?
        } else {
            File::open(path)?
        };

        println!("valid plot file: {}", plot_file);

        Ok(Plot {
            _account_id: account_id,
            start_nonce: start_nonce,
            nonces: nonces,
            fh: fh,
            read_offset: 0,
            use_direct_io: use_direct_io,
            name: plot_file.to_string(),
        })
    }

    pub fn prepare(&mut self, scoop: u32) -> io::Result<u64> {
        self.read_offset = 0;
        let nonces = self.nonces;
        let mut seek_start = scoop as u64 * nonces as u64 * SCOOP_SIZE;

        if self.use_direct_io {
            let r = seek_start % 512;
            if r != 0 {
                seek_start += 512 - r;
                self.read_offset = 512 - r;
            }
        }

        self.fh.seek(SeekFrom::Start(seek_start))
    }

    pub fn read(&mut self, bs: &mut Vec<u8>, scoop: u32) -> Result<(usize, u64, bool), io::Error> {
        let read_offset = self.read_offset;
        let buffer_cap = bs.capacity();
        let start_nonce = self.start_nonce + self.read_offset / 64;

        let (bytes_to_read, finished) = if read_offset as usize + buffer_cap
            >= (SCOOP_SIZE * self.nonces) as usize
        {
            let mut bytes_to_read = (SCOOP_SIZE * self.nonces) as usize - self.read_offset as usize;
            if self.use_direct_io {
                let r = bytes_to_read % 512;
                if r != 0 {
                    bytes_to_read -= r;
                }
            }

            (bytes_to_read, true)
        } else {
            (buffer_cap as usize, false)
        };

        let offset = self.read_offset;
        let nonces = self.nonces;
        let seek_addr =
            SeekFrom::Start(offset as u64 + scoop as u64 * nonces as u64 * SCOOP_SIZE);
        self.fh.seek(seek_addr)?;

        self.fh.read_exact(&mut bs[0..bytes_to_read])?;
		
		self.read_offset += bytes_to_read as u64;

        Ok((bytes_to_read, start_nonce, finished))
    }
}
