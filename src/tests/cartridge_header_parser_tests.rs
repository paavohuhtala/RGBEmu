use std;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::emulation::cartridge::CartridgeHeader;

fn load_roms() -> Vec<(String, Vec<u8>)> {
    let path = Path::new("./test_roms/");
    let mut roms = vec![];
    for entry_result in std::fs::read_dir(path).unwrap() {
        let entry = entry_result.unwrap();
        if entry.file_type().unwrap().is_file() {
            let path = entry.path();
            let mut file = vec![];
            File::open(path.clone())
                .unwrap()
                .read_to_end(&mut file)
                .unwrap();
            roms.push((
                path.file_name().unwrap().to_str().unwrap().to_string(),
                file
            ));
        }
    }
    roms
}

#[test]
#[ignore]
fn load_headers() {
    let roms = load_roms();
    for (name, data) in roms {
        println!("{:?}", name);
        CartridgeHeader::parse(&data);
    }
}
