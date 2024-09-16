use std::path::PathBuf;

use blrs::info::LocalBuild;

#[tokio::main]
async fn main() {}

pub fn test_reading() {
    let daily_builds_path = PathBuf::from("/mnt/Bigass/Blender/linux-builds/daily/");
    println!["Reading builds..."];
    let builds: Vec<LocalBuild> = daily_builds_path
        .read_dir()
        .unwrap()
        .filter_map(|folder| {
            println!["{:?}", folder];

            let folder = folder.ok()?;

            if folder.file_type().ok()?.is_dir() {
                let folder_path = PathBuf::from(folder.file_name());
                let full_path = daily_builds_path.join(&folder_path);
                let build_info_path = &full_path.join(".build_info");
                println!["{:?}", build_info_path];
                match LocalBuild::read_exact(build_info_path) {
                    Ok(build) => {
                        println!["Read build: {:#?}", &build];

                        Some(build)
                    }
                    Err(e) => {
                        println!["Failed to read build: {:?}", e];
                        // Read the build to generate a LocalBlendBuild
                        println!["Attempting to read the build for information"];
                        let executable = full_path.join("blender");
                        let local_build = LocalBuild::generate_from_exe(&executable)
                            .inspect_err(|e| println!("Error: {:?}", e));

                        match local_build {
                            Ok(build) => {
                                // save the file to disk so that we can read it later
                                println!["Saving build... {:?}", &build];
                                let r = build.write();
                                println!["{:?}", r];

                                Some(build)
                            }
                            Err(e) => {
                                println!["Error: {:?}", e];
                                None
                            }
                        }
                    }
                }
            } else {
                None
            }
        })
        .collect();

    println!["{:#?}", builds];
}
