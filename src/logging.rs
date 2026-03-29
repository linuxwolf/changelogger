use std::io::{self, Write};

use env_logger::{Builder, Env, fmt::Formatter};
use log::Level;

pub fn init_logging() {
    let default_level = Level::Info.to_string();
    let env = Env::new().filter_or("CHANGELOGGER_LEVEL", default_level);
    Builder::from_env(env).format(format_log).init();
}

fn format_log(buf: &mut Formatter, rec: &log::Record<'_>) -> io::Result<()> {
    let level = rec.level();
    let style = buf.default_level_style(level);

    writeln!(buf, "{style}{}{style:#}", rec.args())
}
