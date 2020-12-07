use std::env;
use std::collections::HashMap;
use std::fs::read_dir;
use std::fs::read_to_string;
use serde_json::from_str;
use reqwest::header::USER_AGENT;
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
    zonefile_dir : String,
    czds_user: String,
    czds_pass: String,
}


#[derive(Serialize, Deserialize)]
struct Auth {
    accessToken : String,
    message: String
}

fn main() {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    println!("i <3 golang");
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        //probably there's a better way to do this.
        panic!("give me a filename");
    }
    let config_file = &args[1];

    let con = parse_config(config_file);
    check_old_zonefiles(&con);

    match rt.block_on(czds_auth(&con)) {
        Ok(_) => println!("Auth Done"),
        Err(e) => panic!("An error ocurred: {}", e),
    };
    
    
}

async fn czds_auth(con: &Config) -> Result<(),Box<dyn std::error::Error>> {
// {"username": "jvolizka@jvol.gay", "password": "hunter2"}   
    let mut creds = HashMap::new();
    creds.insert("username", &con.czds_user);
    creds.insert("password", &con.czds_pass);

    let client = reqwest::Client::new();

    //I hate this syntax
    let res = client.post("https://account-api.icann.org/api/authenticate")
    .json(&creds)
    .header("Accept", "application/json")
    .header(USER_AGENT , "jvolsbane / 0.0.1 kill_me")
    .send()
    .await?;

    if !res.status().is_success() {
        panic!("There is a problem with response :/ status code : {:?}" , res.text().await?)
    } 
    
    let key: Auth = res.json().await?; 
    println!("{}", key.message);
    Ok(())
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