use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "FOnline Overlay", author = "qthree <qthree3@gmail.com>")]
pub(crate) struct Config {
    /// Sets the client process id
    #[structopt(long)]
    pub(crate) pid: Option<u32>,
    /// Wait for client window
    #[structopt(short)]
    pub(crate) wait: bool,
    /// Sets the web server address
    #[structopt(name = "URL")]
    pub(crate) url: String,
}
impl Config {
    pub(super) fn from_args() -> Self {
        StructOpt::from_args()
    }
}
