use {
    clap::{crate_description, crate_name, App, Arg, ArgMatches},
    solana_sdk::{
        fee_calculator::FeeRateGovernor,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair},
    },
    std::{net::SocketAddr, process::exit, time::Duration},
};

const NUM_LAMPORTS_PER_ACCOUNT_DEFAULT: u64 = solana_sdk::native_token::LAMPORTS_PER_SOL;

/// Holds the configuration for a single run of the benchmark
pub struct Config {
    pub entrypoint_addr: SocketAddr,
    pub id: Keypair,
    pub threads: usize,
    pub num_nodes: usize,
    pub duration: Duration,
    pub tx_count: usize,
    pub keypair_multiplier: usize,
    pub thread_batch_sleep_ms: usize,
    pub sustained: bool,
    pub client_ids_and_stake_file: String,
    pub write_to_client_file: bool,
    pub read_from_client_file: bool,
    pub target_lamports_per_signature: u64,
    pub multi_client: bool,
    pub num_lamports_per_account: u64,
    pub target_slots_per_epoch: u64,
    pub target_node: Option<Pubkey>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            entrypoint_addr: SocketAddr::from(([127, 0, 0, 1], 8001)),
            id: Keypair::new(),
            threads: 4,
            num_nodes: 1,
            duration: Duration::new(std::u64::MAX, 0),
            tx_count: 50_000,
            keypair_multiplier: 8,
            thread_batch_sleep_ms: 1000,
            sustained: false,
            client_ids_and_stake_file: String::new(),
            write_to_client_file: false,
            read_from_client_file: false,
            target_lamports_per_signature: FeeRateGovernor::default().target_lamports_per_signature,
            multi_client: true,
            num_lamports_per_account: NUM_LAMPORTS_PER_ACCOUNT_DEFAULT,
            target_slots_per_epoch: 0,
            target_node: None,
        }
    }
}

/// Defines and builds the CLI args for a run of the benchmark
pub fn build_args<'a, 'b>(version: &'b str) -> App<'a, 'b> {
    App::new(crate_name!()).about(crate_description!())
        .version(version)
        .arg(
            Arg::with_name("entrypoint")
                .short("n")
                .long("entrypoint")
                .value_name("HOST:PORT")
                .takes_value(true)
                .help("Rendezvous with the cluster at this entry point; defaults to 127.0.0.1:8001"),
        )
        .arg(
            Arg::with_name("faucet")
                .short("d")
                .long("faucet")
                .value_name("HOST:PORT")
                .takes_value(true)
                .hidden(true)
                .help("Deprecated. BenchTps no longer queries the faucet directly"),
        )
        .arg(
            Arg::with_name("identity")
                .short("i")
                .long("identity")
                .value_name("PATH")
                .takes_value(true)
                .help("File containing a client identity (keypair)"),
        )
        .arg(
            Arg::with_name("num-nodes")
                .short("N")
                .long("num-nodes")
                .value_name("NUM")
                .takes_value(true)
                .help("Wait for NUM nodes to converge"),
        )
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .value_name("NUM")
                .takes_value(true)
                .help("Number of threads"),
        )
        .arg(
            Arg::with_name("duration")
                .long("duration")
                .value_name("SECS")
                .takes_value(true)
                .help("Seconds to run benchmark, then exit; default is forever"),
        )
        .arg(
            Arg::with_name("sustained")
                .long("sustained")
                .help("Use sustained performance mode vs. peak mode. This overlaps the tx generation with transfers."),
        )
        .arg(
            Arg::with_name("no-multi-client")
                .long("no-multi-client")
                .help("Disable multi-client support, only transact with the entrypoint."),
        )
        .arg(
            Arg::with_name("target_node")
                .long("target-node")
                .requires("no-multi-client")
                .takes_value(true)
                .value_name("PUBKEY")
                .help("Specify an exact node to send transactions to."),
        )
        .arg(
            Arg::with_name("tx_count")
                .long("tx_count")
                .value_name("NUM")
                .takes_value(true)
                .help("Number of transactions to send per batch")
        )
        .arg(
            Arg::with_name("keypair_multiplier")
                .long("keypair-multiplier")
                .value_name("NUM")
                .takes_value(true)
                .help("Multiply by transaction count to determine number of keypairs to create")
        )
        .arg(
            Arg::with_name("thread-batch-sleep-ms")
                .short("z")
                .long("thread-batch-sleep-ms")
                .value_name("NUM")
                .takes_value(true)
                .help("Per-thread-per-iteration sleep in ms"),
        )
        .arg(
            Arg::with_name("write-client-keys")
                .long("write-client-keys")
                .value_name("FILENAME")
                .takes_value(true)
                .help("Generate client keys and stakes and write the list to YAML file"),
        )
        .arg(
            Arg::with_name("read-client-keys")
                .long("read-client-keys")
                .value_name("FILENAME")
                .takes_value(true)
                .help("Read client keys and stakes from the YAML file"),
        )
        .arg(
            Arg::with_name("target_lamports_per_signature")
                .long("target-lamports-per-signature")
                .value_name("LAMPORTS")
                .takes_value(true)
                .help(
                    "The cost in lamports that the cluster will charge for signature \
                     verification when the cluster is operating at target-signatures-per-slot",
                ),
        )
        .arg(
            Arg::with_name("num_lamports_per_account")
                .long("num-lamports-per-account")
                .value_name("LAMPORTS")
                .takes_value(true)
                .help(
                    "Number of lamports per account.",
                ),
        )
        .arg(
            Arg::with_name("target_slots_per_epoch")
                .long("target-slots-per-epoch")
                .value_name("SLOTS")
                .takes_value(true)
                .help(
                    "Wait until epochs are this many slots long.",
                ),
        )
}

/// Parses a clap `ArgMatches` structure into a `Config`
/// # Arguments
/// * `matches` - command line arguments parsed by clap
/// # Panics
/// Panics if there is trouble parsing any of the arguments
pub fn extract_args(matches: &ArgMatches) -> Config {
    let mut args = Config::default();

    if let Some(addr) = matches.value_of("entrypoint") {
        args.entrypoint_addr = solana_net_utils::parse_host_port(addr).unwrap_or_else(|e| {
            eprintln!("failed to parse entrypoint address: {}", e);
            exit(1)
        });
    }

    if matches.is_present("identity") {
        args.id = read_keypair_file(matches.value_of("identity").unwrap())
            .expect("can't read client identity");
    }

    if let Some(t) = matches.value_of("threads") {
        args.threads = t.to_string().parse().expect("can't parse threads");
    }

    if let Some(n) = matches.value_of("num-nodes") {
        args.num_nodes = n.to_string().parse().expect("can't parse num-nodes");
    }

    if let Some(duration) = matches.value_of("duration") {
        args.duration = Duration::new(
            duration.to_string().parse().expect("can't parse duration"),
            0,
        );
    }

    if let Some(s) = matches.value_of("tx_count") {
        args.tx_count = s.to_string().parse().expect("can't parse tx_count");
    }

    if let Some(s) = matches.value_of("keypair_multiplier") {
        args.keypair_multiplier = s
            .to_string()
            .parse()
            .expect("can't parse keypair-multiplier");
        assert!(args.keypair_multiplier >= 2);
    }

    if let Some(t) = matches.value_of("thread-batch-sleep-ms") {
        args.thread_batch_sleep_ms = t
            .to_string()
            .parse()
            .expect("can't parse thread-batch-sleep-ms");
    }

    args.sustained = matches.is_present("sustained");

    if let Some(s) = matches.value_of("write-client-keys") {
        args.write_to_client_file = true;
        args.client_ids_and_stake_file = s.to_string();
    }

    if let Some(s) = matches.value_of("read-client-keys") {
        assert!(!args.write_to_client_file);
        args.read_from_client_file = true;
        args.client_ids_and_stake_file = s.to_string();
    }

    if let Some(v) = matches.value_of("target_lamports_per_signature") {
        args.target_lamports_per_signature = v.to_string().parse().expect("can't parse lamports");
    }

    args.multi_client = !matches.is_present("no-multi-client");
    args.target_node = matches
        .value_of("target_node")
        .map(|target_str| target_str.parse().unwrap());

    if let Some(v) = matches.value_of("num_lamports_per_account") {
        args.num_lamports_per_account = v.to_string().parse().expect("can't parse lamports");
    }

    if let Some(t) = matches.value_of("target_slots_per_epoch") {
        args.target_slots_per_epoch = t
            .to_string()
            .parse()
            .expect("can't parse target slots per epoch");
    }

    args
}
