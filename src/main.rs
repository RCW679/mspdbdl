use std::fs::File;
use std::process::ExitCode;
use std::str::FromStr;
use std::{env, error::Error, io, fs, path::PathBuf};
use exe::Buffer;
use exe::pe::{VecPE, Castable};
use exe::types::DebugDirectory;
use reqwest::{blocking, Url};
use glob::glob;
#[repr(C, packed)]
#[derive(Castable)]
struct DDRaw {
    magic: [u8; 4],
    guid: [u8; 16],
    age: u32,
    name: [u8; 255],
}
struct URLResult(String, String);
fn get_url(image: &VecPE) -> Result<URLResult, Box<dyn Error>> {
    let dir = DebugDirectory::parse(image)?;
    let dd = image.get_ref::<DDRaw>(dir.pointer_to_raw_data.into())?;
    let debug_name = String::from_utf8(dd.name.as_slice()
                        [0..(dd.name.into_iter().position(|x| x == 0)
                        .ok_or(io::Error::new(io::ErrorKind::InvalidData,
                            "Invalid string"))? as usize)].to_vec())?;
    let wg = dd.guid;
    let debug_guid = hex::encode([wg[0x3], wg[0x2], wg[0x1], wg[0x0],
                                  wg[0x5], wg[0x4], wg[0x7], wg[0x6],
                                  wg[0x8], wg[0x9], wg[0xA], wg[0xB],
                                  wg[0xC], wg[0xD], wg[0xE], wg[0xF]]).to_uppercase();
    let debug_age = dd.age;
    return Result::Ok(URLResult(format!("https://msdl.microsoft.com/download/symbols/{}/{}{}/{}",
                                         debug_name, debug_guid, debug_age, debug_name), debug_name));
}
fn main() -> Result<ExitCode, Box<dyn Error>> {
    println!("mspdbdl v1.0 - a PDB symbol downloader");
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: mspdbdl file_or_glob [file_or_glob [...]]");
        return Result::Ok(1.into());
    }
    let client = blocking::Client::new();
    for i in 1..args.len() {
        for gpath in glob(&args[i])? {
            let gp = gpath?;
            let fname = gp.to_str().unwrap();
            println!("Processing {}", fname);
            let image = VecPE::from_disk_file(fname)?;
            let url = get_url(&image)?;
            println!("PDB URL: {}", &url.0);
            let destpath = fs::canonicalize(PathBuf::from(fname))?.parent()
                                .unwrap().join(PathBuf::from(&url.1)).to_str().unwrap().to_string();
            println!("Downloading {} to {}", &url.1, &destpath);
            let mut writer = File::create(destpath)?;
            client.get(Url::from_str(url.0.as_str())?).send()?.copy_to(&mut writer)?;
        }
    }
    return Result::Ok(0.into());
}