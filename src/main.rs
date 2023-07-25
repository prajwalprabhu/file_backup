use clap::Parser;
use regex::Regex;
use std::fs::{copy, read_dir, DirBuilder, DirEntry, File};
use std::path::PathBuf;
#[derive(Debug)]
enum FileType {
    Dir(Vec<DirEntry>),
    File(File),
}
impl FileType {
    fn files(self) -> Option<Vec<DirEntry>> {
        if let FileType::Dir(files) = self {
            Some(files)
        } else {
            None
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("./"),help=String::from("Source Path"))]
    src_path: String,

    #[arg(short, long,help = String::from("Destination Path"))]
    dest_path: String,
    #[arg(short,long,help=String::from("Recursively Copy"),default_value_t=false)]
    recursive: bool,

    #[arg(short,long,help=String::from("Recursively Copy"),default_value_t=false)]
    force: bool,

    #[arg(short,long,help=String::from("Copy Only Updated"),default_value_t=false)]
    update: bool,
    #[arg(short,long,help=String::from("Files or folders to ignore (regex) coma separated"),default_value_t=String::new())]
    exclude: String,
    // #[arg(short, long, default_value_t = 1)]
    // count: u8,
}
impl Args {
    fn run(&self) -> Result<(), String> {
        let src = self.open(&self.src_path, false)?;
        let dest = self.open(&self.dest_path, true)?;
        if let FileType::File(_) = dest {
            return Err(String::from("Destination must be a directory "));
        }
        let dest_files = dest.files().unwrap();
        match src {
            FileType::Dir(files) => {
                for f in files.iter() {
                    //check for duplicate file names
                    let file_name = String::from(f.file_name().to_str().unwrap());
                    let meta_data = f.metadata().unwrap();
                    let from = self.create_src_path(&file_name);
                    let to = self.create_dest_path(&file_name);
                    if self
                        .exclude
                        .split(',')
                        .filter_map(|r| Regex::new(&format!("^{r}$")).ok())
                        .any(|p| p.captures(&file_name).is_some())
                    {
                        println!("Exclude {:?}", file_name);
                        continue;
                    }
                    let dest_file = dest_files
                        .iter()
                        .find(|ff| ff.file_name().eq(&f.file_name()));
                    if !self.update && !self.force && dest_file.is_some() {
                        println!(
                            "Override not allowed use --force or -f to copy forcefully {:?}",
                            &f.file_name()
                        );
                        continue;
                    }
                    //skip unchanged files
                    else if meta_data.is_file()
                        && dest_file.is_some()
                        && dest_file
                            .unwrap()
                            .metadata()
                            .unwrap()
                            .modified()
                            .unwrap()
                            .cmp(&meta_data.modified().unwrap())
                            .is_eq()
                    {
                        continue;
                    } else if meta_data.is_file() {
                        self.copy(&file_name, &from, &to)?;
                    } else if meta_data.is_dir() && self.recursive {
                        Args {
                            src_path: String::from(from.to_str().unwrap()),
                            dest_path: String::from(to.to_str().unwrap()),
                            recursive: self.recursive,
                            force: self.force,
                            update: self.update,
                            exclude: self.exclude.clone(),
                        }
                        .run()?
                    }
                }
                // copy(, to)
            }
            FileType::File(_) => {
                let file_name = self.src_path.clone();
                let from = self.create_src_path(&file_name);
                let to = self.create_dest_path(&file_name);
                // let meta_data = f.metadata().unwrap();
                let dest_file = dest_files
                    .iter()
                    .find(|ff| String::from(ff.file_name().to_str().unwrap()).eq(&file_name));
                // println!("{:?}", dest_file);
                dbg!(&dest_file);
                if !self.update && !self.force && dest_file.is_some() {
                    println!(
                        "Override not allowed use --force or -f to copy forcefully {:?}",
                        &file_name
                    );
                    return Ok(());
                }

                self.copy(&file_name, &from, &to)?;
                println!("File {}", self.src_path);
            }
        }
        Ok(())
    }
    fn copy(&self, file_name: &str, from: &PathBuf, to: &PathBuf) -> Result<(), String> {
        println!(
            "Copying {:?} from {:?} {:?}",
            file_name,
            from.as_os_str(),
            to.as_os_str()
        );
        copy(from, to)
            .map_err(|e| format!("Error Coping file {:?}", e))
            .map(|_| ())
    }
    fn create_path(&self, path: &str, name: &str) -> PathBuf {
        let mut from = PathBuf::new();
        from.push(path);
        from.push(name);
        from
    }
    fn create_src_path(&self, name: &str) -> PathBuf {
        self.create_path(&self.src_path, name)
    }
    fn create_dest_path(&self, name: &str) -> PathBuf {
        self.create_path(&self.dest_path, name)
    }
    fn open(&self, path: &str, make: bool) -> Result<FileType, String> {
        if let Ok(file) = File::open(path) {
            if file.metadata().unwrap().is_dir() {
                Ok(FileType::Dir(
                    read_dir(path).unwrap().map(|d| d.unwrap()).collect(),
                ))
            } else {
                Ok(FileType::File(file))
            }
        } else if make {
            DirBuilder::new()
                .recursive(true)
                .create(path)
                .map_err(|e| format!("Error Creating Folder {:?}", e))?;
            Ok(FileType::Dir(
                read_dir(path).unwrap().map(|d| d.unwrap()).collect(),
            ))
        } else {
            Err(format!("{} path does not exists .", self.src_path))
        }
    }
}
fn main() {
    let args = Args::parse();
    if let Err(err) = args.run() {
        eprintln!("Oops! Error occurred  {err}");
    }
    // for _ in 0..args.count {
    //     println!("Hello {}!", args.src_dir)
    // }
}
