use std::io::Read;
use std::io::Write;
use futures::future::join_all;
use std::env;
use std::collections::HashMap;
use std::fs::read_dir;
use std::fs;
use std::fs::read_to_string;
use serde_json::from_str;
use reqwest::header::USER_AGENT;
use std::fs::File;
use flate2::read::GzDecoder;
//use std::io;
//use std::io::copy;
//use std::error::Error;
//use futures::future;
use tokio::task::JoinHandle;
//use std::io::Read;
//use std::process;
//use serde::Deserialize;
//use serde_json::Result;
use serde_derive::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
#[derive(Clone)]
struct Config {
    tlds: Vec<String> ,//The tld vector 
    zonefile_dir : String,
    czds_user: String,
    czds_pass: String,
    czds_base_url: String,
    czds_auth_url: String
}

#[derive(Serialize, Deserialize)]
struct Auth {
    accessToken : String,
    message: String
}

fn main() {
    let token: Auth;
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
        Ok(key) => token = key,
        Err(e) => panic!("An error ocurred: {}", e),
    };
    let zncl_con = con.clone();
    match rt.block_on(download_zonefiles(zncl_con, token)) {
        Ok(_) => println!("Download done",),
        Err(e) => panic!("An error ocurred: {}", e),
    };    
    let uzcl_con = con.clone();
    match rt.block_on(unzip_and_compare(uzcl_con)) {
        Ok(_) => println!("compare done",),
        Err(e) => panic!("An error ocurred: {}", e),
    };  
    
}

async fn unzip_and_compare(con: Config) -> Result<(),Box<dyn std::error::Error>> {
    let mut tasks: Vec<JoinHandle<Result<(), ()>>>= vec![];
    
    for tld in con.tlds {
        println!("started processing {}" , &tld);
        let filedir = con.zonefile_dir.clone();
        tasks.push(tokio::spawn(async move {

        let new_file = match File::open(format!("{}/{}.txt.gz" ,filedir , &tld)) {
            Err(why) => panic!("couldn't open {}: {}", tld, why),
            Ok(file) => file,
        };

        let mut new_s = String::new();
        let mut new_dec = GzDecoder::new(new_file);
        new_dec.read_to_string(&mut new_s).unwrap();
        let new_splt = new_s.split("\n");
        let new_str_vec: Vec<&str> = new_splt.collect();

        let old_file = match File::open(format!("{}/{}.txt.gz.old" , filedir , &tld)) {
            Err(why) => panic!("couldn't open {}: {}", tld, why),
            Ok(file) => file,
        };

        let mut old_s = String::new();
        let mut old_dec = GzDecoder::new(old_file);
        old_dec.read_to_string(&mut old_s).unwrap();
        let old_splt = old_s.split("\n");
        let old_str_vec: Vec<&str> = old_splt.collect();

        let mut diff: Vec<&str> = Vec::new();

        for dom in new_str_vec {
            if !old_str_vec.contains(&dom) {
                diff.push(dom);
            }
        }

        println!("{} {}" , tld , diff.len());

        let diff_path = format!("{}/{}.diff", filedir , &tld); 
        let mut diff_file = match File::create(&diff_path) {
            Err(why) => panic!("couldn't open {}",  why),
            Ok(file) => file,
        };

        for i in diff {
            write!(diff_file,"{} \n" , i);
        }
        fs::rename(format!("{}/{}.txt.gz" ,&filedir,  &tld ) , format!("{}/{}.txt.gz.old" ,&filedir,  &tld)).expect(&format!( "couldn't rename {}.txt.gz to {}.txt.gz.old" , &tld  , &tld ));
        Ok(())
        }));
    }
    join_all(tasks).await;
    Ok(())
}


async fn download_zonefiles(con: Config , key: Auth) -> Result<(),Box<dyn std::error::Error>> {
    
    let mut tasks: Vec<JoinHandle<Result<(), ()>>>= vec![];
    
    for tld in con.tlds {
        let path = format!("{}/czds/downloads/{}.zone", &con.czds_base_url , &tld);
        //let path = format!("http://127.0.0.1/{}.txt.gz", &tld );
        let filedir = con.zonefile_dir.clone();
        let token = key.accessToken.clone();
        let client = reqwest::Client::new();
        let agent =  "jvolsbane / 0.0.1 kill_me";

        // Create a Tokio task for each url
        tasks.push(tokio::spawn(async move {
            println!("started downloading {}" , path);
            match client.get(&path)
            .header("Accept" , "application/json")
            .header(USER_AGENT, agent)
            .header("Authorization" , format!("Bearer {}", token))
            .send()
            .await
             {

                Ok(resp) => {
                    if !resp.status().is_success(){
                        println!("ERROR downloading {}", resp.status())
                    }
                
                    match resp.bytes().await {
                        Ok(data) => {
                            
                            println!("RESPONSE: {} bytes from {}", data.len(), path);
                            
                            let filename = format!("{}{}.txt.gz" , filedir  , &tld );
                            let mut file = match File::create(filename) {
                                Err(why) => panic!("couldn't create {}", why),
                                Ok(file) => file,
                            };

                            file.write_all(&data).expect("failed to write file");
                            
                        }
                        Err(e) => println!("ERROR reading {} - {}", path , e),
                    }
                }
                Err(e) => println!("ERROR downloading {} - {}", path , e),
            }
            Ok(())
        }));
    }

    // Wait for them all to finish
    println!("Started {} tasks. Waiting...", tasks.len());
    join_all(tasks).await;

    Ok(())

}

async fn czds_auth(con: &Config) -> Result<Auth,Box<dyn std::error::Error>> {
// {"username": "jvolizka@jvol.gay", "password": "hunter2"}   
    let mut creds = HashMap::new();
    let agent =  "jvolsbane / 0.0.1 kill_me";
    creds.insert("username", &con.czds_user);
    creds.insert("password", &con.czds_pass);

    let client = reqwest::Client::new();

    //I hate this syntax

    let res = client.post(&format!("{}/api/authenticate" , &con.czds_auth_url))
    .json(&creds)
    .header("Accept", "application/json")
    .header(USER_AGENT , agent)
    .send()
    .await?;

    if !res.status().is_success() {
        panic!("There is a problem with response :/ status code : {:?}" , res.text().await?)
    } 

    let key: Auth = res.json().await?; 
    println!("{}", key.message);

    let links = client.get(&format!("{}/czds/downloads/links" , &con.czds_base_url))
    .header("Accept" , "application/json")
    .header(USER_AGENT,agent)
    .header("Authorization" , format!("Bearer {}", key.accessToken))
    .send()
    .await?;

    let stuff: Vec<String> = links.json().await?; 
    // check if we can actually download the tld list
    for tld in &con.tlds {
        if !stuff.contains(&format!("{}/czds/downloads/{}.zone" ,&con.czds_base_url , &tld)) {
            panic!("You don't have permission to download {} zonefile" , &tld);
        }
    }
    
    println!("Permission granted");
    Ok(key)
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