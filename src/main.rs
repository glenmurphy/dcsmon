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

fn parse_cookie(headers: &HeaderMap) -> String {
    let mut cookies = vec![];
    for (key, value) in headers.iter() {
        if key == "set-cookie" {
            cookies.push(value.to_str().unwrap())
        }
    }
    cookies.join(", ")
}

async fn login(username: String, password: String) -> Result<String, &'static str> {
    if username.is_empty() || password.is_empty() {
        return Err("No username or password");
    }

    let mut login_headers = HeaderMap::new();
    login_headers.insert("content-type", "application/x-www-form-urlencoded".parse().unwrap());

    let client = reqwest::Client::new();
    let res = client.post("https://www.digitalcombatsimulator.com/en/")
        .headers(login_headers)
        .body(format!("AUTH_FORM=Y&TYPE=AUTH&backurl=%2Fen%2F&USER_LOGIN={}&USER_PASSWORD={}&USER_REMEMBER=Y&Login=Authorize", username, password))
        .send().await
        .unwrap();

    let cookies = parse_cookie(res.headers());
    if !cookies.contains("BITRIX_SM_UIDL=") {
        return Err("username/password incorrect");
    }

    Ok(cookies)
}

async fn get_servers(cookies: String) -> Result<Servers, &'static str> {
    let mut headers = HeaderMap::new();
    headers.insert(reqwest::header::COOKIE, cookies.parse().unwrap());

    let client = reqwest::Client::new();
    let servers = client.get("https://www.digitalcombatsimulator.com/en/personal/server/?ajax=y")
        .headers(headers)
        .send()
        .await.unwrap()
        .json::<Servers>()
        .await.unwrap();

    Ok(servers)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let filter = args.filter.to_lowercase();

    // Do login
    let cookies = login(args.username, args.password).await;
    if let Err(msg) = cookies {
        println!("\x1b[31mLogin failed: {}\x1b[0m", msg);
        return Ok(());
    }
 
    // Request server list
    match get_servers(cookies.unwrap()).await {
        Ok(servers) => display_servers(&servers, &filter),
        Err(msg) => println!("\x1b[31mFailed to get server list: {}\x1b[0m", msg)
    }

    Ok(())
}