extern crate log;
extern crate log4rs;
use config::Cfg;

use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::filter::threshold::ThresholdFilter;

pub fn init_logger(cfg: &Cfg) -> log4rs::Handle {
    
	let level_console = match (&cfg.console_log_level).as_str() {
        "Trace" => log::LevelFilter::Trace,
        "Debug" => log::LevelFilter::Debug,
        "Info" => log::LevelFilter::Info,
        "Warn" => log::LevelFilter::Warn,
        "Error" => log::LevelFilter::Error,
		"Off" => log::LevelFilter::Off,
        _ => log::LevelFilter::Info,
    };
	
	let level_logile = match (&cfg.console_log_level).as_str() {
        "Trace" => log::LevelFilter::Trace,
        "Debug" => log::LevelFilter::Debug,
        "Info" => log::LevelFilter::Info,
        "Warn" => log::LevelFilter::Warn,
        "Error" => log::LevelFilter::Error,
		"Off" => log::LevelFilter::Off,
        _ => log::LevelFilter::Warn,
    };
	
	let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{({d(%H:%M:%S)} [{l}]):16.16} {m}{n}",
        )))
        .build();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{({d(%Y-%m-%d %H:%M:%S)} [{l}]):26.26} {m}{n}",
        )))
        .build("log/scavenger.log")
        .unwrap();

    let config = Config::builder()
        .appender(
			Appender::builder()
				.filter(Box::new(ThresholdFilter::new(level_console)))
				.build("stdout", Box::new(stdout)))

			.appender(
				Appender::builder()
					.filter(Box::new(ThresholdFilter::new(level_logile)))
					.build("logfile", Box::new(logfile)),
        )
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap()
}
