use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Error},
    net::Ipv4Addr,
    path::Path,
    process::Command,
    thread,
};

pub fn determine_config(
    args: Vec<String>,
) -> Result<
    (
        Ipv4Addr,
        u16,
        String,
        String,
        String,
        Option<String>,
        String,
        String,
        bool,
        Verbosity,
    ),
    Error,
> {
    // Process args
    // Check environment
    // Make sure that the public directory exists, if not, check with user, then download.
    // Check if a config file exists, if not, check with user, then download.

    let mut address = None;
    let mut port = None;
    let mut web_index = None;
    let mut root_location = None;
    let mut gen_args = None;
    let mut jar_name = None;
    let mut min_ram = None;
    let mut max_ram = None;
    let mut web_log = None;
    let mut verbosity = None;
    let mut download_public = None;
    let mut download_config = None;

    match args.len() {
        0 => panic!("No file specified, this might not be needed"),
        1 => {} // No args set nothing
        _ => {
            // There exist args to be parsed
            for (index, arg) in args.iter().enumerate() {
                if index % 2 == 1 && index != args.len() - 1 && index > 0 {
                    // all args should fall on an odd index as all require a second parameter
                    match arg.as_str() {
                        "--location" | "-l" => {
                            root_location = Some(verify_location(args[index + 1].clone()))
                        }
                        "--address" | "-a" => {
                            address = Some(verify_address(args[index + 1].clone()))
                        }
                        "--port" | "-p" => port = Some(verify_port(args[index + 1].clone())),
                        "--web_index" | "-i" => web_index = Some(args[index + 1].clone()),
                        "--jar" | "-j" => jar_name = Some(verify_jar(args[index + 1].clone())),
                        "--min" | "-m" => min_ram = Some(verify_min_ram(args[index + 1].clone())),
                        "--max" | "-M" => max_ram = Some(verify_max_ram(args[index + 1].clone())),
                        "--download_webdir" | "-w" => {
                            download_public = Some(verify_download_web(args[index + 1].clone()))
                        }
                        "--download_config" | "-c" => {
                            // true or false
                            download_config = Some(match args[index + 1].as_str() {
                                "true" => true,
                                "false" => false,
                                _ => panic!(
                                    "Boolean not found for download configuration, found: {}",
                                    args[index + 1]
                                ),
                            });
                        }
                        "--log_web" | "-o" => {
                            web_log = Some(verify_web_log(args[index + 1].clone()))
                        }
                        "--verbosity" | "-v" => {
                            verbosity = Some(verify_verbosity(args[index + 1].clone()))
                        }
                        "--args" | "-x" => {
                            gen_args = Some(verify_general_args(args[index + 1].clone()))
                        }
                        _ => panic!("Invalid parameter found, found: {}", *arg),
                    }
                } else if index % 2 == 1 && index != 0 {
                    // There is not a following arg
                    panic!(
                        "Make sure that all args are followed by a value. {} is missing a value.",
                        arg
                    );
                }
            }
        }
    };
    // Command Line Arguments should have been parsed and error checked
    if download_config == None {
        // If not set, do not download
        download_config = Some(false);
    }
    // Parse through a config file if it exists
    let config_path = Path::new("config.conf");
    if !config_path.exists() {
        // No file exists check if a default one should be downloaded
        if download_config.unwrap() {
            let mut config_curl = Command::new("curl")
                .arg("-s")
                .arg("https://raw.githubusercontent.com/nuhtan/minecraft_monitor/main/config.conf")
                .arg("-O")
                .spawn()
                .expect("Error?");
            config_curl.wait()?;
            println!("Config file downloaded");
        }
    }

    // read through config file, notify of parsing and formatting errors
    let reader = BufReader::new(File::open("config.conf").unwrap());
    for (index, line) in reader.lines().enumerate() {
        let line = line.unwrap(); // Not sure what errors could happen here
        if line != "" && !(&line[0..1] == "#") {
            // Comments and blank lines are ignored
            let equal = match line.find("=") {
                    Some(loc) => loc,
                    None => panic!("Line within configuration file does not contain '=', all settings should contain this character. Line: {} contains an error", index)
                };
            // println!("0..=: {}, =..:{}", &line[0..equal], &line[equal + 2..line.len() - 1]);
            match &line[0..equal] {
                "server_location" => {
                    if root_location == None {
                        root_location =
                            Some(verify_location(line[equal + 2..line.len() - 1].to_string()));
                    }
                }
                "webserver_address" => {
                    if address == None {
                        address = Some(verify_address(line[equal + 2..line.len() - 1].to_string()));
                    }
                }
                "webserver_port" => {
                    if port == None {
                        port = Some(verify_port(line[equal + 2..line.len() - 1].to_string()));
                    }
                }
                "webserver_index" => {
                    if web_index == None {
                        web_index = Some(line[equal + 2..line.len() - 1].to_string());
                    }
                }
                "generic_args" => {
                    if gen_args == None {
                        gen_args = Some(verify_general_args(
                            line[equal + 2..line.len() - 1].to_string(),
                        ));
                    }
                }
                "server_jar" => {
                    if jar_name == None {
                        jar_name = Some(verify_jar(line[equal + 2..line.len() - 1].to_string()));
                    }
                }
                "minimum_ram" => {
                    if min_ram == None {
                        min_ram = Some(verify_min_ram(line[equal + 2..line.len() - 1].to_string()));
                    }
                }
                "maximum_ram" => {
                    if max_ram == None {
                        max_ram = Some(verify_max_ram(line[equal + 2..line.len() - 1].to_string()));
                    }
                }
                "download_public" => {
                    if download_public == None {
                        download_public = Some(verify_download_web(
                            line[equal + 2..line.len() - 1].to_string(),
                        ));
                    }
                }
                "log_web" => {
                    if web_log == None {
                        web_log = Some(verify_web_log(line[equal + 2..line.len() - 1].to_string()));
                    }
                }
                "verbosity" => {
                    if verbosity == None {
                        verbosity = Some(verify_verbosity(
                            line[equal + 2..line.len() - 1].to_string(),
                        ));
                    }
                }
                _ => {
                    panic!("Unexpected config found on line: {}, found: {}. If this line was intended to be a comment please prefix the line with a '#'", index, line)
                }
            }
        }
    }

    // If a parameter has not been set then use the default
    if root_location == None {
        root_location = Some("./server".to_string());
    }
    if address == None {
        address = Some("127.0.0.1".parse::<Ipv4Addr>().unwrap());
    }
    if port == None {
        port = Some(8000);
    }
    if web_index == None {
        web_index = Some("/html/home.html".to_string());
    }
    if gen_args == None {
        gen_args = Some(None);
    }
    if jar_name == None {
        jar_name = Some("minecraft_server.1.16.4.jar".to_string());
    }
    if min_ram == None {
        min_ram = Some("1G".to_string());
    }
    if max_ram == None {
        max_ram = Some("2G".to_string());
    }
    if download_public == None {
        download_public = Some(false);
    }
    if web_log == None {
        web_log = Some(false);
    }
    if verbosity == None {
        verbosity = Some(Verbosity::None);
    }

    // Download and determine if the web dir exists
    if !Path::new("../public").exists() {
        // No folder exists check if a default one should be downloaded
        if download_public.unwrap() {
            // Create public directory
            match fs::create_dir("public") {
                Ok(_) => {
                    let mut config_curl = Command::new("curl").arg("-s").arg("https://raw.githubusercontent.com/nuhtan/minecraft_monitor/main/public/manifest.json").arg("-O").spawn().expect("Error?");
                    config_curl.wait()?;
                    println!("Manifest file downloaded");
                    // Read manifest
                    let manifest = fs::read_to_string("manifest.json").unwrap();
                    if manifest == "404: Not Found" {
                        panic!("Manifest was improperly downloaded, report to repo.");
                    }
                    // Parse manifest
                    let mut file_list = Vec::new();
                    let mut files = manifest[1..manifest.len() - 1].replace("\n    ", "");
                    while files.len() > 0 {
                        let end = match files.find("\",\"") {
                            Some(ending) => ending,
                            None => match files.find("\"\n") {
                                Some(ending) => ending,
                                None => panic!(
                                    "Error in manifest file! Check that brackets are closed."
                                ),
                            },
                        };
                        file_list.push(files[1..end].to_string());
                        files = files[end + 2..].to_string();
                    }
                    // Files are downloaded
                    let mut thread_list = Vec::new();
                    for file in file_list {
                        let mut ext = "";
                        for (i, c) in file.chars().into_iter().rev().enumerate() {
                            if c == '.' {
                                ext = &file[file.len() - 1 - i..];
                                break;
                            }
                        }
                        if ext == "" {
                            panic!("File missing extension, found: {:?}", ext);
                        }
                        let folder = match ext {
                            ".html" => "html",
                            ".css" => "css",
                            ".js" => "javascript",
                            _ => "images",
                        };
                        if !Path::new(&format!("public/{}", folder)).exists() {
                            fs::create_dir(format!("public/{}", folder)).unwrap();
                        }
                        let handle = thread::spawn(move || {
                            let mut file_curl = Command::new("curl").arg("-s").arg(format!("https://raw.githubusercontent.com/nuhtan/minecraft_monitor/main/public/{}/{}", folder, file)).arg("-o").arg(format!("public/{}/{}", folder, file)).spawn().expect("Error?");
                            file_curl.wait().unwrap();
                            println!("Downloaded web server file: {}/{}", folder, file);
                        });
                        thread_list.push(handle);
                    }
                    for thread in thread_list {
                        thread.join().unwrap();
                    }
                }
                Err(_) => {
                    println!("A web directory already exists, forcing the application to try to download a copy from the repo does nothing. If this is not intentional remove any directories with a name of 'public'.");
                }
            }
        }
    }

    Ok((
        address.unwrap(),
        port.unwrap(),
        web_index.unwrap(),
        root_location.unwrap(),
        jar_name.unwrap(),
        gen_args.unwrap(),
        min_ram.unwrap(),
        max_ram.unwrap(),
        web_log.unwrap(),
        verbosity.unwrap(),
    ))
}

#[derive(Clone, PartialEq)]
pub enum Verbosity {
    None,
    Mine,
    Web,
    MineWeb,
}

fn verify_address(arg: String) -> Ipv4Addr {
    match arg.parse::<Ipv4Addr>() {
        Ok(addr) => return addr,
        Err(_) => panic!("Invalid ip address, found: {}", arg),
    };
}

fn verify_port(arg: String) -> u16 {
    match arg.parse::<u16>() {
        Ok(p) => return p,
        Err(_) => panic!("Invalid port, found {}", arg),
    };
}

fn verify_location(arg: String) -> String {
    let path = Path::new(&arg);
    if path.exists() {
        return arg;
    } else {
        panic!(
            "Specified directory does not exist, path: {}",
            path.display()
        );
    }
}
fn verify_jar(arg: String) -> String {
    let path = Path::new(&arg);
    let extension = path.extension();
    match extension {
            Some(ext) => {
                if ext == "jar" {
                    return arg;
                } else {
                    panic!("The file specified should be a .jar, found: {:?}", ext);
                }
            },
            None => panic!("The specified file either has no name or has no extension. Expecting a .jar extension.")
        }
}

fn verify_general_args(arg: String) -> Option<String> {
    return match arg.as_str() {
        "off" => None,
        _ => Some(arg),
    };
}

fn verify_min_ram(arg: String) -> String {
    println!("{}", arg);
    let data_size = arg.chars().last().unwrap();
    match data_size {
        'K' | 'M' | 'G' => {
            return match arg[0..arg.len() - 1].parse::<u32>() {
                Ok(_) => arg,
                Err(_) => panic!(
                    "Invalid number found for minimum allocated ram, found: {}",
                    &arg[0..arg.len() - 1]
                ),
            };
        }
        _ => panic!(
            "Invalid data size found for minimum allocated ram, found: {:?}",
            data_size
        ),
    }
}

fn verify_max_ram(arg: String) -> String {
    let data_size = arg.chars().last().unwrap();
    match data_size {
        'K' | 'M' | 'G' => {
            return match arg[0..arg.len() - 1].parse::<u32>() {
                Ok(_) => arg,
                Err(_) => panic!(
                    "Invalid number found for maximum alloram, found: {}",
                    &arg[0..arg.len() - 1]
                ),
            };
        }
        _ => panic!(
            "Invalid data size found for maximum allocated ram, found: {}",
            data_size
        ),
    };
}

fn verify_web_log(arg: String) -> bool {
    return match arg.as_str() {
        "true" => true,
        "false" => false,
        _ => panic!("Boolean not found for web log, found: {}", arg),
    };
}

fn verify_verbosity(arg: String) -> Verbosity {
    return match arg.as_str() {
        "none" => Verbosity::None,
        "mine" => Verbosity::Mine,
        "web" => Verbosity::Web,
        "mineweb" => Verbosity::MineWeb,
        _ => panic!("Invalid parameter for verbosity found, found: {}", arg),
    };
}

fn verify_download_web(arg: String) -> bool {
    return match arg.as_str() {
        "true" => true,
        "false" => false,
        _ => panic!(
            "Boolean not found for download web directory, found: {}",
            arg
        ),
    };
}
