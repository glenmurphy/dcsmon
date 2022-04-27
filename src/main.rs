use reqwest::header::HeaderMap;
use serde::{Deserialize};
use clap::Parser;

/**
 * Structs for serde to be able to deserealize the json
 */
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

/**
 * Config for clap's command line argument thingy
 */
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

/**
 * Turns the DCS goobledegook into something usable in a terminal; won't yet correct
 * for the spaces DCS adds to allow line breaks on its website
 */
fn sanitize_name(name: &str) -> String {
    let mut fixed = name.replace(|c: char| !c.is_ascii(), "");
    fixed = fixed.replace("&amp;", "&");
    fixed = fixed.replace("&gt;", ">");
    fixed = fixed.replace("&lt;", "<");
    fixed.trim().to_string()
}

fn display_servers(servers: &Servers, filter : &String) {
    println!("\n\x1b[93m{:36.36}   {:30.30}   {}\x1b[0m", "Name", "Mission", "Players");

    // TODO: display MY_SERVERS so server admins can use this as a simple tool

    for server in &servers.SERVERS {
        if server.NAME.to_lowercase().contains(filter) {
            println!("{:36.36}   {:30.30}   {}", 
                sanitize_name(&server.NAME),
                sanitize_name(&server.MISSION_NAME),
                server.PLAYERS.parse::<i32>().unwrap() - 1);
        }
    }
}

/**
 * As get_all("set-cookie") doesn't work, we have to manually parse the separate
 * set-cookie lines into a single cookie string.
 */
fn parse_cookie(headers: &HeaderMap) -> String {
    let mut cookies = vec![];
    for (key, value) in headers.iter() {
        if key == "set-cookie" {
            cookies.push(value.to_str().unwrap())
        }
    }
    cookies.join(", ")
}

/**
 * Gets a login cookie from the DCS website
 */
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

/**
 * Gets the current list of servers from the DCS website
 */
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

/**
 * We don't really need async for this, especially with the blocking library available,
 * but it's nice to have it for the future (if we want to display progress), and it 
 * doesn't impact the binary size
 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let cookies = login(args.username, args.password).await;
    if let Err(msg) = cookies {
        println!("\x1b[31mLogin failed: {}\x1b[0m", msg);
        return Ok(());
    }
 
    match get_servers(cookies.unwrap()).await {
        Ok(servers) => display_servers(&servers, &args.filter.to_lowercase()),
        Err(msg) => println!("\x1b[31mFailed to get server list: {}\x1b[0m", msg)
    }

    Ok(())
}