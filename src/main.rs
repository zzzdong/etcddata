use std::path::{Path, PathBuf};

use anyhow::Result;
use etcdv3client::EtcdClient;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "etcddata", about = "etcd data dump and restore")]
enum Opt {
    #[structopt(name = "dump")]
    Dump(DumpCmd),
    #[structopt(name = "restore")]
    Restore(RestoreCmd),
    #[structopt(name = "print")]
    Print(PrintCmd),
    #[structopt(name = "read")]
    Read(ReadCmd),
}

#[derive(StructOpt, Debug)]
struct EtcdOpts {
    #[structopt(long = "host", short = "h")]
    host: String,
    #[structopt(long = "port", short = "p")]
    port: u16,
    #[structopt(long = "user", short = "u")]
    user: Option<String>,
    #[structopt(long = "password", short = "P")]
    password: Option<String>,
    #[structopt(long = "prefix")]
    prefix: Option<String>,
}

#[derive(StructOpt, Debug)]
struct DumpCmd {
    #[structopt(flatten)]
    etcd_opts: EtcdOpts,
    #[structopt(long = "dir", short = "d", parse(from_os_str))]
    dir: PathBuf,
    #[structopt(long = "all")]
    all: bool,
}

#[derive(StructOpt, Debug)]
struct RestoreCmd {
    #[structopt(flatten)]
    etcd_opts: EtcdOpts,
    #[structopt(long = "dir", short = "d", parse(from_os_str))]
    dir: PathBuf,
}

#[derive(StructOpt, Debug)]
struct PrintCmd {
    #[structopt(flatten)]
    etcd_opts: EtcdOpts,
    #[structopt(long = "all")]
    all: bool,
}

#[derive(StructOpt, Debug)]
struct ReadCmd {
    #[structopt(long = "dir", short = "d", parse(from_os_str))]
    dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let opt = Opt::from_args();

    match opt {
        Opt::Dump(o) => dump_data(&o.etcd_opts, &o.dir, o.all).await?,
        Opt::Restore(o) => restore_data(&o.etcd_opts, &o.dir).await?,
        Opt::Print(o) => print_data(&o.etcd_opts, o.all).await?,
        Opt::Read(o) => read_data(&o.dir)?,
    }

    Ok(())
}

async fn connect_etcd(etcd_opts: &EtcdOpts) -> Result<EtcdClient> {
    let mut auth: Option<(String, String)> = None;
    if let Some(user) = &etcd_opts.user {
        auth = Some((user.clone(), etcd_opts.password.clone().unwrap()));
    }

    let endpoint = format!("http://{}:{}", etcd_opts.host, etcd_opts.port);

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
