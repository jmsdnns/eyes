use clap::{Command, Arg, ArgAction, ArgMatches, value_parser};
use futures::{stream, StreamExt};
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::net::TcpStream;

struct ScanConfig {
    target_ip: IpAddr,
    ports: Vec<u16>,
    verbose: bool,
    concurrency: usize,
    timeout: u64
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = init_cli();
    let args = cli.get_matches();
    let scan_config = init_scan_config(args);

    scan(&scan_config).await;

    Ok(())
}

fn init_cli() -> Command {
    let cli = Command::new("eyes")
        .arg(
            Arg::new("target")
                .help("The IP to scan")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("verbose")
                .help("Display detailed information")
                .long("verbose")
                .short('v')
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("ports")
                .help("List of ports to scan")
                .long("ports")
                .short('p')
                .default_value("1-1024"),
        )
        .arg(
            Arg::new("concurrency")
                .help("Number of simultaneous scanners")
                .long("concurrency")
                .short('c')
                .value_parser(value_parser!(usize))
                .default_value("1000"),
        )
        .arg(
            Arg::new("timeout")
                .help("Connection timeout")
                .long("timeout")
                .short('t')
                .value_parser(value_parser!(u64))
                .default_value("3"),
        )
        .arg_required_else_help(true);

    return cli
}

fn init_scan_config(args: ArgMatches) -> ScanConfig {
    // Prepare host address
    let target = args.get_one::<String>("target").expect("required");
    let sockaddr: SocketAddr = format!("{}:0", target).parse().unwrap();
    let target_ip = sockaddr.ip();

    // Optional CLI params
    let verbose = args.get_flag("verbose");
    let concurrency = match args.get_one::<usize>("concurrency") {
        Some(c) => usize::from(*c),
        _ => 1000
    };
    let ports = match args.get_one::<String>("ports") {
        Some(p) => parse_ports(String::from(p)),
        _ => parse_ports(String::from("1-1024"))
    };
    let timeout = match args.get_one::<u64>("timeout") {
        Some(t) => u64::from(*t),
        _ => 3
    };

    let sc = ScanConfig {
        target_ip,
        ports,
        verbose,
        concurrency,
        timeout,
    };

    // Log the config params
    if sc.verbose {
        println!("[eyes] Scanning {} ports on {}", sc.ports.len(), sc.target_ip);
        println!("[eyes] Concurrency: {}", sc.concurrency);
        println!("[eyes] Timeout: {}", sc.timeout);
    }

    sc
}

fn parse_ports(ports_arg: String) -> Vec<u16> {
    let mut ports: Vec<u16> = Vec::new();

    if ports_arg.contains(",") {
        let ps = ports_arg.split(",");
        for p in ps {
            if p.contains("-") {
                let range: Vec<u16> = p.split('-').map(|x: &str| x.parse::<u16>().unwrap()).collect();
                ports.extend(range[0]..=range[1]);
            }
            else {
                if let Ok(p) = p.parse::<u16>() {
                    ports.push(p);
                }
            }
        }
    }
    else if ports_arg.contains("-") {
        let range: Vec<u16> = ports_arg.split('-').map(|x: &str| x.parse::<u16>().unwrap()).collect();
        ports.extend(range[0]..=range[1]);
    }
    else if let Ok(p) = ports_arg.parse::<u16>() {
        ports.push(p);
    }
    else {
        ports.extend(1..=1024);
    }    

    ports
}

async fn scan(sc: &ScanConfig) {
    let port_stream = stream::iter(Box::new(sc.ports.clone()).into_iter());

    port_stream
        .for_each_concurrent(sc.concurrency, |port| open_port(sc, port))
        .await;

    println!("[eyes] Finished scan");
}

async fn open_port(c: &ScanConfig, port: u16) {
    let timeout = Duration::from_secs(c.timeout);
    let socket_address = SocketAddr::new(c.target_ip.clone(), port);

    match tokio::time::timeout(timeout, TcpStream::connect(&socket_address)).await {
        Ok(Ok(_)) => println!("{}: open", port),
        _ => if c.verbose { println!("{}: closed", port); }  
    }
}
