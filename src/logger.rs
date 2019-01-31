use crate::config::Cfg;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;

fn to_log_level(s: &str, default: log::LevelFilter) -> log::LevelFilter {
    match s.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        "off" => log::LevelFilter::Off,
        _ => default,
    }
}

pub fn init_logger(cfg: &Cfg) -> log4rs::Handle {
    let level_console = to_log_level(&cfg.console_log_level, log::LevelFilter::Info);
    let level_logfile = to_log_level(&cfg.logfile_log_level, log::LevelFilter::Warn);
    let mut console_log_pattern = if cfg.show_progress {
        "\r".to_owned()
    } else {
        "".to_owned()
    };
    console_log_pattern.push_str(&cfg.console_log_pattern);
    let mut logfile_log_pattern = if cfg.show_progress {
        "\r".to_owned()
    } else {
        "".to_owned()
    };
    logfile_log_pattern.push_str(&cfg.logfile_log_pattern);

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(&console_log_pattern)))
        .build();

    let roller = FixedWindowRoller::builder()
        .base(1)
        .build("log/scavenger.{}.log", cfg.logfile_max_count)
        .unwrap();
    let trigger = SizeTrigger::new(&cfg.logfile_max_size * 1024 * 1024);
    let policy = Box::new(CompoundPolicy::new(Box::new(trigger), Box::new(roller)));

    let config = if level_logfile == log::LevelFilter::Off {
        Config::builder()
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(level_console)))
                    .build("stdout", Box::new(stdout)),
            )
            .build(Root::builder().appender("stdout").build(LevelFilter::Info))
            .unwrap()
    } else {
        let logfile = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(&logfile_log_pattern)))
            .build("log/scavenger.1.log", policy)
            .unwrap();
        Config::builder()
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(level_console)))
                    .build("stdout", Box::new(stdout)),
            )
            .appender(
                Appender::builder()
                    .filter(Box::new(ThresholdFilter::new(level_logfile)))
                    .build("logfile", Box::new(logfile)),
            )
            .build(
                Root::builder()
                    .appender("stdout")
                    .appender("logfile")
                    .build(LevelFilter::Trace),
            )
            .unwrap()
    };
    log4rs::init_config(config).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_log_level() {
        assert_eq!(
            to_log_level("Trace", log::LevelFilter::Error),
            log::LevelFilter::Trace
        );
        assert_eq!(
            to_log_level("Foo", log::LevelFilter::Error),
            log::LevelFilter::Error
        );
        assert_eq!(
            to_log_level("DEBUG", log::LevelFilter::Error),
            log::LevelFilter::Debug
        );
        assert_eq!(
            to_log_level("InFo", log::LevelFilter::Error),
            log::LevelFilter::Info
        );
        assert_eq!(
            to_log_level("eRROR", log::LevelFilter::Info),
            log::LevelFilter::Error
        );
        assert_eq!(
            to_log_level("WARN", log::LevelFilter::Info),
            log::LevelFilter::Warn
        );
        assert_eq!(
            to_log_level("Off", log::LevelFilter::Info),
            log::LevelFilter::Off
        );
    }

    #[test]
    fn test_init_logger() {
        use crate::config::load_cfg;
        let mut cfg = load_cfg("config.yaml");

        // we dont want to see this during tests
        cfg.console_log_level = log::LevelFilter::Error.to_string();

        let _ = init_logger(&cfg);
        trace!("TRACE");
        debug!("DEBUG");
        info!("INFO");
    }
}
