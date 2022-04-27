use reqwest::header::HeaderMap;
use serde_json::{Value};
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Your DCS username
    #[clap(short)]
    username: String,

    /// Your DCS password
    #[clap(short)]
    password: String,

    /// A server name search filter
    #[clap(short, default_value(""))]
    filter: String,
}

fn get_cookie(headers: &HeaderMap) -> String {
    let mut cookies = vec![];
    for (key, value) in headers.iter() {
        if key == "set-cookie" {
            cookies.push(value.to_str().unwrap())
        }
    }
    cookies.join(", ")
}

fn display_server(server: &Value) {
    let name = server["NAME"].as_str().unwrap().replace(|c: char| !c.is_ascii(), "");
    println!("{:40.40} | {:30.30} | {}", 
        name.trim(),
        server["MISSION_NAME"].as_str().unwrap(),
        server["PLAYERS"].as_str().unwrap().parse::<i32>().unwrap() - 1);
}

fn display_servers(servers: &Value, filter : &String) {
    for server in servers["SERVERS"].as_array().unwrap() {
        let name = server["NAME"].as_str().unwrap().to_lowercase();
        if name.contains(filter) {
            display_server(&server);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let username = args.username;
    let password = args.password;
    let filter = args.filter.to_lowercase();

    if username.is_empty() || password.is_empty() {
        println!("Please provide a username and password");
        return Ok(());
    }

    // Do login
    let mut login_headers = HeaderMap::new();
    //login_headers.insert("accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9".parse().unwrap());
    //login_headers.insert("accept-language", "en-US,en;q=0.9".parse().unwrap());
    //login_headers.insert("cache-control", "max-age=0".parse().unwrap());
    login_headers.insert("content-type", "application/x-www-form-urlencoded".parse().unwrap());

    let client = reqwest::Client::new();
    let res = client.post("https://www.digitalcombatsimulator.com/en/")
        .headers(login_headers)
        .body(format!("AUTH_FORM=Y&TYPE=AUTH&backurl=%2Fen%2F&USER_LOGIN={}&USER_PASSWORD={}&USER_REMEMBER=Y&Login=Authorize", username, password))
        .send()
        .await?;
    
    // Parse cookies
    let cookie_str = get_cookie(res.headers());
    if !cookie_str.contains("BITRIX_SM_UIDL=") {
        println!("Failed to log in!");
        return Ok(());
    }

    // Request server list
    let mut server_headers = HeaderMap::new();
    server_headers.insert(reqwest::header::COOKIE, cookie_str.parse().unwrap());   

    let req = client.get("https://www.digitalcombatsimulator.com/en/personal/server/?ajax=y")
        .headers(server_headers)
        .send()
        .await?
        .json::<Value>()
        .await?;

    display_servers(&req, &filter);

    Ok(())
}