use std::path::{Path, PathBuf};

use etcdv3client::{kv::SimpleKVClient, EtcdV3Client};
use failure::Error;
use rocksdb::{IteratorMode, Options, DB};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "etcddata", about = "etcd data dump and restore")]
enum Opt {
    #[structopt(name = "dump")]
    Dump(DumpCmd),
    #[structopt(name = "restore")]
    Restore(RestoreCmd),
    #[structopt(name = "debug")]
    Debug(DebugCmd),
}

#[derive(StructOpt, Debug)]
struct DumpCmd {
    #[structopt(long = "all")]
    all: bool,
    #[structopt(name = "host", short = "h")]
    host: String,
    #[structopt(name = "port", short = "p")]
    port: u16,
    #[structopt(name = "DIR", short = "d", parse(from_os_str))]
    dir: PathBuf,
}

#[derive(StructOpt, Debug)]
struct RestoreCmd {
    #[structopt(long = "host", short = "h")]
    host: String,
    #[structopt(long = "port", short = "p")]
    port: u16,
    #[structopt(name = "DIR", short = "d", parse(from_os_str))]
    dir: PathBuf,
}

#[derive(StructOpt, Debug)]
struct DebugCmd {
    #[structopt(name = "DIR", short = "d", parse(from_os_str))]
    dir: PathBuf,
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let opt = Opt::from_args();

    match opt {
        Opt::Dump(o) => dump_data(&o.host, o.port, &o.dir, o.all)?,
        Opt::Restore(o) => restore_data(&o.host, o.port, &o.dir)?,
        Opt::Debug(o) => debug_data(&o.dir)?,
    }

    Ok(())
}

fn dump_data(host: &str, port: u16, path: &Path, all: bool) -> Result<(), Error> {
    let etcd_client = EtcdV3Client::new(host, port).unwrap();
    let client = etcd_client.new_simple_kv();

    let db = DB::open_default(path).unwrap();

    let kvs = client.get_all()?;
    for kv in kvs {
        if all || kv.lease == 0 {
            db.put(kv.key, kv.value).unwrap();
        }
    }

    db.flush()?;

    Ok(())
}

fn restore_data(host: &str, port: u16, path: &Path) -> Result<(), Error> {
    let etcd_client = EtcdV3Client::new(host, port).unwrap();
    let client = etcd_client.new_simple_kv();

    let db = DB::open_default(path).unwrap();
    let iter = db.iterator(IteratorMode::Start);

    for (key, value) in iter {
        client.put_bytes_kv(key.to_vec(), value.to_vec())?;
    }

    Ok(())
}

fn debug_data(path: &Path) -> Result<(), Error> {
    let db = DB::open_default(path).unwrap();
    let iter = db.iterator(IteratorMode::Start);

    for (key, value) in iter {
        println!(
            "{} => {}",
            String::from_utf8_lossy(&key),
            String::from_utf8_lossy(&value)
        );
    }

    Ok(())
}
