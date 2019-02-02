use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "tcp_spy")]
struct CliOptions {
    #[structopt(short = "e", long = "expose", default_value = "9000")]
    e: u16,

    #[structopt(short = "h", long = "host", default_value = "127.0.0.1")]
    host: String,

    #[structopt(short = "p", long = "port", default_value = "8080")]
    port: u16,
}

#[derive(Debug, Clone)]
pub struct Opt {
    pub source: String,
    pub target: String,
}

pub fn from_args() -> Opt {
    let opt = CliOptions::from_args();
    Opt {
        source: format!("127.0.0.1:{}", opt.e),
        target: parse_host(opt.host, opt.port),
    }
}

fn parse_host(t: String, port: u16) -> String {
    let mut t = String::from(t);
    let t = if t.starts_with("https://") {
        t.split_off(8)
    } else if t.starts_with("http://") {
        t.split_off(7)
    } else {
        t
    };
    if t.contains(":") {
        t.to_string()
    } else {
        format!("{}:{}", t, port)
    }
}
