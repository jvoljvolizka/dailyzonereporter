use std::env;
use std::fs::read_dir;
use std::fs::read_to_string;
use serde_json::from_str;
//use std::fs::File;
//use std::io;
//use std::io::Read;
//use std::process;
//use serde::Deserialize;
//use serde_json::Result;
use serde_derive::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
struct Config {
    tlds: Vec<String> ,//The tld vector 
    zonefile_dir : String
}

fn main() {
    println!("i <3 golang");
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        //probably there's a better way to do this.
        panic!("give me a filename");
    }
    let config_file = &args[1];

    let con = parse_config(config_file);
    check_old_zonefiles(&con);
    
    
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}
fn check_old_zonefiles(con: &Config) {
    // read the zonefile dir 
    let paths = read_dir(&con.zonefile_dir).expect("Directory check err");
    let mut files: Vec<String> = Vec::new();

    for path in paths {// get every ok filename to a vector
        if let Ok(path) = path {
            let file = format!("{:?}" , path.file_name());
            files.push(file);
        }
    }
    for tld in &con.tlds { // check if every .old archive is in the zonefile dir 
        //if you can't beat them join them
        let check = format!("\"{}.txt.gz.old\"", tld); 

        if !files.contains(&check){
            panic!("Could not find {} file" , &check);
        }
    }
    
}

//Parse config.json
fn parse_config(filename: &str)  -> Config {
    // i have no idea how to deal with two different error types
    let contents = read_to_string(filename).expect("File read err");
    let con: Config = from_str(&contents).expect("Json deser err");
    return con
}