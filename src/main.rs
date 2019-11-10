use std::env;
use std::thread;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use filetime::FileTime;

fn main() {
    let args: Vec<String> = env::args().collect();
    let args = parse_args(&args);

    let mut src_path = PathBuf::new();
    src_path.push(args.src_path);
    let mut dest_path = PathBuf::new();
    dest_path.push(args.dest_path);

    sync_folder(&src_path, &dest_path);
    println!("done");
}

fn sync_folder(src_dir: &PathBuf, dest_dir: &PathBuf) -> std::io::Result<()> {
    let mut dest_dir_path = dest_dir.clone();
    let mut children = vec![];

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();

        let result = check_dir_existence(&path, &dest_dir);
        dest_dir_path.push(path.file_name().unwrap());

        if path.is_dir() {
            // Check for directory existence
            if !result.unwrap() {
                fs::create_dir(dest_dir_path.as_path());
            }
                
            sync_folder(&path, &dest_dir_path);
        }
        else {
            if !result.unwrap() {
                let dest_path = dest_dir_path.clone();
                children.push(thread::spawn(move || {
                    copy_file(&path, &dest_path);
                    println!("File copied: {:}", &dest_path.display());
                }));
            }
        }

        dest_dir_path.pop();
    }

    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }

    Ok(())
}

fn check_dir_existence(src_dir: &Path, dest_dir: &Path) -> std::io::Result<bool> {
    let dir_to_check = src_dir.file_name();
    let mut found = false; 

    for entry in fs::read_dir(dest_dir)? {
        let entry = entry?;
        let path = entry.path();
        let dir = path.file_name();

        if dir == dir_to_check {
            if path.is_dir() {
                found = true;
            }
            else {
                found = check_file_sync(&src_dir, &path);
            }
            
            break;
        }
    }    

    Ok(found)
}

fn check_file_sync(src_dir: &Path, dest_dir: &Path) -> bool {
    let metadata = fs::metadata(&src_dir).unwrap();
    let src_time = FileTime::from_last_modification_time(&metadata);

    let metadata = fs::metadata(&dest_dir).unwrap();
    let dest_time = FileTime::from_last_modification_time(&metadata);

    dest_time >= src_time
}

fn copy_file(src_file_path: &Path, dest_file_path: &Path) -> std::io::Result<()> {
    fs::copy(src_file_path, dest_file_path)?;
    Ok(())
}

fn parse_args(args: &[String]) -> Arguments {
    let src_path = args[1].clone();
    let dest_path = args[2].clone();

    Arguments { src_path, dest_path }
}

struct Arguments {
    src_path: String,
    dest_path: String,
}