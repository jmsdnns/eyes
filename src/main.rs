use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use clap::{Command, Arg, ArgAction, ArgMatches, value_parser};
use futures::{stream, StreamExt};
use tokio::net::TcpStream;

/// All of the values required for a single pool of scanners
struct ScanConfig {
    /// IP Address for target being scanned
    target_ip: IpAddr,
    /// Collection of unique port addresses
    ports: Vec<u16>,
    /// Number of simultaneous scanners
    concurrency: usize,
    /// Connection timeout
    timeout: u64,
    /// Toggle for verbose output
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Convert CLI args into `ScanConfig`
    let cli = init_cli();
    let args = cli.get_matches();
    let scan_config = init_scan_config(args);

    // Begin scan
    scan(&scan_config).await;

    Ok(())
}

/// Creates and configures the CLI argument structure
fn init_cli() -> Command {
    let cli = Command::new("eyes")
        // <target>
        .arg(
            Arg::new("target")
                .help("The IP to scan")
                .required(true)
                .index(1),
        )
        // --verbose
        .arg(
            Arg::new("verbose")
                .help("Display detailed information")
                .long("verbose")
                .short('v')
                .action(ArgAction::SetTrue),
        )
        // --ports <arg>
        .arg(
            Arg::new("ports")
                .help("List of ports to scan")
                .long("ports")
                .short('p')
                .default_value("1-1024"),
        )
        // --concurrency <arg>
        .arg(
            Arg::new("concurrency")
                .help("Number of simultaneous scanners")
                .long("concurrency")
                .short('c')
                .value_parser(value_parser!(usize))
                .default_value("1000"),
        )
        // --timeout <arg>
        .arg(
            Arg::new("timeout")
                .help("Connection timeout")
                .long("timeout")
                .short('t')
                .value_parser(value_parser!(u64))
                .default_value("3"),
        )
        // print help if no cli args are given
        .arg_required_else_help(true);

    return cli
}

/// Converts CLI args into a `ScanConfig` instance
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

    // Build config
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

/// Converts user-provided description of ports into `Vec` of unique numbers
/// 
///  Ports can be expressed as:
///    * list of numbers: `eyes <target> -p 22,80,1336`
///    * range of numbers: `eyes <target> -p 22-80`
///    * mix of both: `eyes <target> -p 22,80,8000-8099,443,8443,3000-3443`
fn parse_ports(ports_arg: String) -> Vec<u16> {
    let mut ports: Vec<u16> = Vec::new();

    // List of ports found: `x,y,z`
    if ports_arg.contains(",") {
        let ps = ports_arg.split(",");
        for p in ps {
            // List item is port range: `x-z`
            if p.contains("-") {
                let range: Vec<u16> = p.split('-').map(|x: &str| x.parse::<u16>().unwrap()).collect();
                ports.extend(range[0]..=range[1]);
            }
            // List item is single port: `x`
            else {
                if let Ok(p) = p.parse::<u16>() {
                    ports.push(p);
                }
            }
        }
    }
    // Range of ports found: `x-z`
    else if ports_arg.contains("-") {
        let range: Vec<u16> = ports_arg.split('-').map(|x: &str| x.parse::<u16>().unwrap()).collect();
        ports.extend(range[0]..=range[1]);
    }
    // Single port found: `x`
    else if let Ok(p) = ports_arg.parse::<u16>() {
        ports.push(p);
    }
    // Default to range of ports: `1-1024`
    else {
        ports.extend(1..=1024);
    }    

    ports
}

/// Runs a scan based on params found in `ScanConfig`
async fn scan(sc: &ScanConfig) {
    let port_stream = stream::iter(Box::new(sc.ports.clone()).into_iter());

    // Spawn 1 coroutine per port scanned
    port_stream
        .for_each_concurrent(sc.concurrency, |port| open_port(sc, port))
        .await;

    println!("[eyes] Finished scan");
}

/// Attempts to open a connection to target on the provided `port`
async fn open_port(c: &ScanConfig, port: u16) {
    let timeout = Duration::from_secs(c.timeout);
    let socket_address = SocketAddr::new(c.target_ip.clone(), port);

    match tokio::time::timeout(timeout, TcpStream::connect(&socket_address)).await {
        Ok(Ok(_)) => println!("{}: open", port),
        _ => if c.verbose { println!("{}: closed", port); }  
    }
}
