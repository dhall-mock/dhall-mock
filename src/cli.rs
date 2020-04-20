use structopt::StructOpt;

#[derive(StructOpt, Debug, Clone)]
#[structopt(name = "dhall-mock")]
pub struct CliOpt {
    /// Dhall configuration files to parse
    #[structopt(required = true)]
    pub configuration_files: Vec<String>,
    /// http binding for server
    #[structopt(short, long, default_value = "0.0.0.0:8088")]
    pub http_bind: String,
    #[structopt(short, long, default_value = "0.0.0.0:8089")]
    pub admin_http_bind: String,
}

pub fn load_cli_args() -> CliOpt {
    CliOpt::from_args()
}
