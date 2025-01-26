use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let target = env::var("TARGET").expect("TARGET not set");
    let profile = env::var("PROFILE").expect("PROFILE not set");

    let source_path = Path::new(&manifest_dir).join("GiiGa");
    let dest_path = Path::new(&manifest_dir)
        .join("target")
        .join(target)
        .join(profile)
        .join("GiiGa");

    if !source_path.exists() {
        panic!("GiiGa directory not found in project root");
    }

    copy_dir(&source_path, &dest_path).expect("Failed to copy GiiGa directory");
}

fn copy_dir(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let target = dst.join(entry.file_name());
        
        if file_type.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}