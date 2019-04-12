use std::env;
use std::fs;
use std::io;
use std::io::{Read, Write};
use std::path;

use byteorder::ByteOrder;
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
    let mut file = iowrap::Eof::new(io::BufReader::new(file));
    confirm_header_present(&mut file)?;

    let mut out = io::BufWriter::new(tempfile::NamedTempFile::new_in(parent)?);
    out.write_all(&HEADER)?;

    while !file.eof()? {
        let len = file.read_u32::<BE>()?;
        out.write_u32::<BE>(len)?;

        let mut chunk_type = [0u8; 4];
        file.read_exact(&mut chunk_type)?;
        out.write_all(&chunk_type)?;

        println!("{:?}: {}", String::from_utf8_lossy(&chunk_type), len);

        let len_with_crc = u64::from(len) + 4;
        io::copy(&mut (&mut file).take(len_with_crc), &mut out)?;
    }

    out.into_inner()?.persist(path)?;

    Ok(())
}

fn confirm_header_present<R: Read>(mut file: R) -> Result<(), Error> {
    let mut buf = [0u8; 8];
    file.read_exact(&mut buf)
        .with_context(|_| err_msg("reading header"))?;
    ensure!(HEADER == buf, "invalid header, not a png file");
    Ok(())
}
