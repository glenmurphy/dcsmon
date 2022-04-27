use reqwest::header::HeaderMap;
use serde::{Deserialize};
use clap::Parser;

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Server {
    NAME: String,
    MISSION_NAME: String,
    PLAYERS: String,

    //IP_ADDRESS: String,
    //PORT: String,
    //MISSION_TIME: String,
    //PLAYERS_MAX: String,
    //PASSWORD: String,
    //DESCRIPTION: String,
    //DCS_VERSION: String,
    //MISSION_TIME_FORMATTED: String,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Servers {
    SERVERS : Vec<Server>,

    //SERVERS_MAX_COUNT: i32,
    //SERVERS_MAX_DATE: String,
    //PLAYERS_COUNT: i32,
    //MY_SERVERS : Vec<Server>
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Your DCS username (required)
    #[clap(short)]
    username: String,

    /// Your DCS password (required)
    #[clap(short)]
    password: String,

    /// Optional server name search filter
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

fn sanitize_name(name: &str) -> String {
    let mut fixed = name.replace(|c: char| !c.is_ascii(), "");
    fixed = fixed.replace("&amp;", "&");
    fixed = fixed.replace("&gt;", ">");
    fixed = fixed.replace("&lt;", "<");
    fixed.trim().to_string()
}

fn display_servers(servers: &Servers, filter : &String) {
    println!("\n\x1b[93m{:36.36}   {:30.30}   {}\x1b[0m", "Name", "Mission", "Players");

    for server in &servers.SERVERS {
        if server.NAME.to_lowercase().contains(filter) {
            println!("{:36.36}   {:30.30}   {}", 
                sanitize_name(&server.NAME),
                sanitize_name(&server.MISSION_NAME),
                server.PLAYERS.parse::<i32>().unwrap() - 1);
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
        .json::<Servers>()
        .await?;

    display_servers(&req, &filter);

    Ok(())
}