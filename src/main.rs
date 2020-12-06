use std::env;
use std::fs;
//use std::fs::File;
//use std::io;
//use std::io::Read;
//use std::process;
use serde::{Deserialize, Serialize};
//use serde_json::Result;
use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Config {
    tlds: Vec<String> //The tld vector 
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
    
    
}




fn parse_config(filename: &str)  -> Config {
    // i have no idea how to deal with two different error types
    let contents = fs::read_to_string(filename).expect("File read err");

    let con: Config = serde_json::from_str(&contents).expect("Json deser err");

    for line in &con.tlds {
        println!("{}", line)
    }
   
   return con

}