use std::env;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path;

use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use byteorder::BE;
use failure::ensure;
use failure::err_msg;
use failure::format_err;
use failure::Error;
use failure::ResultExt;

const HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];

fn main() -> Result<(), Error> {
    let mut args = env::args_os();
    let us = args.next().unwrap();
    let path = args
        .next()
        .ok_or_else(|| format_err!("usage: {:?} FILE", us))?;
    let path = path::Path::new(&path).canonicalize()?;
    let parent = path
        .parent()
        .ok_or_else(|| err_msg("the root isn't a file!"))?;

    let file = fs::File::open(&path).with_context(|_| err_msg("opening input file"))?;
    let file = io::BufReader::new(file);

    let mut out = io::BufWriter::new(tempfile::NamedTempFile::new_in(parent)?);

    process(file, &mut out)?;

    out.into_inner()?.persist(path)?;

    Ok(())
}

fn process<R: Read, W: Write>(file: R, mut out: W) -> Result<(), Error> {
    let mut file = iowrap::Eof::new(file);

    confirm_header_present(&mut file)?;
    out.write_all(&HEADER)?;

    while !file.eof()? {
        let len = file.read_u32::<BE>()?;
        let len_with_crc = u64::from(len) + 4;

        let mut chunk_type = [0u8; 4];
        file.read_exact(&mut chunk_type)?;
        let critical = chunk_type[0].is_ascii_uppercase();

        let mut data = (&mut file).take(len_with_crc);

        if critical {
            out.write_u32::<BE>(len)?;
            out.write_all(&chunk_type)?;
            io::copy(&mut data, &mut out)?;
        } else {
            io::copy(&mut data, &mut iowrap::Ignore::new())?;
        }
    }

    Ok(())
}

fn confirm_header_present<R: Read>(mut file: R) -> Result<(), Error> {
    let mut buf = [0u8; 8];
    file.read_exact(&mut buf)
        .with_context(|_| err_msg("reading header"))?;
    ensure!(HEADER == buf, "invalid header, not a png file");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::process;

    #[test]
    fn gimp() {
        let mut output = Vec::with_capacity(1024);
        let input = io::Cursor::new(&include_bytes!("../tests/trans-gimp.png")[..]);
        process(input, &mut output).unwrap();
        let expected = &include_bytes!("../tests/trans-gimp-out.png")[..];
        assert_eq!(expected, output.as_slice());
    }
}
