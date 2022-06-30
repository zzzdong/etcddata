use std::path::{Path, PathBuf};

use anyhow::Result;
use etcdv3client::EtcdClient;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "etcddata", about = "etcd data dump and restore")]
enum Args {
    #[clap(name = "dump")]
    Dump(DumpCmd),
    #[clap(name = "restore")]
    Restore(RestoreCmd),
    #[clap(name = "print")]
    Print(PrintCmd),
    #[clap(name = "read")]
    Read(ReadCmd),
}

#[derive(Parser, Debug)]
struct EtcdOpts {
    #[clap(long = "endpoint", short = 'e', default_value = "http://localhost:2379")]
    endpoint: String,
    #[clap(long = "user", short = 'u')]
    user: Option<String>,
    #[clap(long = "password", short = 'P')]
    password: Option<String>,
    #[clap(long = "prefix")]
    prefix: Option<String>,
}

#[derive(Parser, Debug)]
struct DumpCmd {
    #[clap(flatten)]
    etcd_opts: EtcdOpts,
    #[clap(long = "dir", short = 'd', parse(from_os_str))]
    dir: PathBuf,
    #[clap(long = "all")]
    all: bool,
}

#[derive(Parser, Debug)]
struct RestoreCmd {
    #[clap(flatten)]
    etcd_opts: EtcdOpts,
    #[clap(long = "dir", short = 'd', parse(from_os_str))]
    dir: PathBuf,
}

#[derive(Parser, Debug)]
struct PrintCmd {
    #[clap(flatten)]
    etcd_opts: EtcdOpts,
    #[clap(long = "all")]
    all: bool,
}

#[derive(Parser, Debug)]
struct ReadCmd {
    #[clap(long = "dir", short = 'd', parse(from_os_str))]
    dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let opt = Args::from_args();

    match opt {
        Args::Dump(o) => dump_data(&o.etcd_opts, &o.dir, o.all).await?,
        Args::Restore(o) => restore_data(&o.etcd_opts, &o.dir).await?,
        Args::Print(o) => print_data(&o.etcd_opts, o.all).await?,
        Args::Read(o) => read_data(&o.dir)?,
    }

    Ok(())
}

async fn connect_etcd(etcd_opts: &EtcdOpts) -> Result<EtcdClient> {
    let mut auth: Option<(String, String)> = None;

    if etcd_opts.user.is_some() && etcd_opts.password.is_some() {
        auth = Some((
            etcd_opts.user.clone().unwrap(),
            etcd_opts.password.clone().unwrap(),
        ));
    }

    let endpoint = etcd_opts.endpoint.to_string();

    let client = EtcdClient::new(vec![endpoint], auth).await?;

    Ok(client)
}

async fn dump_data(etcd_opts: &EtcdOpts, path: &Path, all: bool) -> Result<()> {
    let mut client = connect_etcd(etcd_opts).await?;

    let tree = sled::open(path)?;
    let mut batch = sled::Batch::default();

    let kvs = match &etcd_opts.prefix {
        Some(p) => client.get_with_prefix(p).await?,
        None => client.all().await?,
    };

    for kv in kvs {
        if all || kv.lease == 0 {
            batch.insert(kv.key, kv.value);
        }
    }

    tree.apply_batch(batch)?;
    tree.flush()?;

    Ok(())
}

async fn restore_data(etcd_opts: &EtcdOpts, path: &Path) -> Result<()> {
    let mut client = connect_etcd(etcd_opts).await?;

    let tree = sled::open(path)?;

    for kv in tree.iter() {
        if let Ok((key, value)) = kv {
            client.put(key.to_vec(), value.to_vec()).await?;
        }
    }

    Ok(())
}

async fn print_data(etcd_opts: &EtcdOpts, all: bool) -> Result<()> {
    let mut client = connect_etcd(etcd_opts).await?;

    let kvs = match &etcd_opts.prefix {
        Some(p) => client.get_with_prefix(p).await?,
        None => client.all().await?,
    };

    for kv in kvs {
        if all || kv.lease == 0 {
            println!(
                "{} => {}",
                String::from_utf8_lossy(&kv.key),
                String::from_utf8_lossy(&kv.value)
            );
        }
    }

    Ok(())
}

fn read_data(path: &Path) -> Result<()> {
    let tree = sled::open(path)?;

    for kv in tree.iter() {
        if let Ok((key, value)) = kv {
            println!(
                "{} => {}",
                String::from_utf8_lossy(&key),
                String::from_utf8_lossy(&value)
            );
        }
    }

    Ok(())
}
