#![allow(dead_code)]
#![allow(clippy::or_fun_call)]
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::{BufReader, Write};
use std::path::PathBuf;
use std::process;
use std::rc::Rc;
use std::str::FromStr;

use anyhow::{bail, Context};
use opentelemetry::sdk::export::trace::stdout;
use tracing::{debug, error, info, span};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

use crate::conf::SettingsSer;
use crate::conf::{into_settings_ref, Settings};
use crate::ui::main_action::{CmdId, InitialAction, MainAction};
use crate::ui::ViewId;

mod cmd;
mod conf;
mod data;
mod db;
mod parsing;
mod ui;
mod util;

fn main() {
    // Create a new OpenTelemetry pipeline
    let tracer = stdout::new_pipeline()
        .with_pretty_print(true)
        .install_simple();

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default().with(telemetry);

    // Trace executed code
    tracing::subscriber::with_default(subscriber, || {
        // Spans will be sent to the configured OpenTelemetry exporter
        let root = span!(tracing::Level::DEBUG, "quarble", work_units = 2);
        let _enter = root.enter();

        main_inner()
    })
    .unwrap();
}

fn main_inner() -> anyhow::Result<()> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    let (settings, args_ref) = match parse_settings(&args_ref) {
        Ok((settings, args_ref)) => (settings, args_ref),
        Err(e) => {
            error!("{:?}", e);
            process::exit(-1);
        }
    };

    let db = db::DB::init(&settings.db_dir)?;

    debug!("{:?}", settings);
    debug!("{:?}", args_ref);

    let initial_action = match args_ref {
        ["day_start"] => InitialAction::Ui(ViewId::FastDayStart),
        ["day_end"] => InitialAction::Ui(ViewId::FastDayEnd),
        ["book"] => InitialAction::Ui(ViewId::BookSingle),
        ["show"] | [] => InitialAction::Ui(ViewId::CurrentDayUi),
        ["print_day"] => InitialAction::Cmd(CmdId::PrintDay),
        unexpected => bail!("Unexpected arguments: {}", unexpected.join(" ")),
    };

    let initial_action = match initial_action {
        InitialAction::Cmd(CmdId::PrintDay) => {
            cmd::print_active_day(db.load_day(settings.active_date)?)
        }
        InitialAction::Ui(id) => id,
    };

    let work_day = if let Some(work_day) = db.load_day(settings.active_date)? {
        work_day
    } else {
        db.new_day(settings.active_date)?
    };

    let main_action = MainAction {
        settings: into_settings_ref(settings),
        initial_view: initial_action,
        db,
        work_day: Rc::new(RefCell::new(work_day)),
    };
    let settings_out = ui::show_ui(main_action);
    let settings_out = settings_out.load();
    if let Err(e) = do_write_settings(&settings_out) {
        error!("{:?}", e);
        process::exit(2);
    }

    Ok(())
}

fn do_write_settings(settings: &Settings) -> anyhow::Result<()> {
    if settings.write_settings {
        let location = settings
            .settings_location
            .as_ref()
            .context("Missing settings location")?;

        info!("Writing settings to {}", location.display());

        if let Some(dir) = location.parent() {
            if !dir.is_dir() {
                std::fs::create_dir_all(dir).with_context(|| {
                    format!("Failed to create settings directory: {}", dir.display())
                })?;
            }
        }

        let to_write = SettingsSer::from_settings(settings);
        let buffer =
            serde_json::to_vec_pretty(&to_write).context("Failed to serialize settings")?;
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(location)
            .context("Cannot open settings for writing")?;
        file.write_all(&buffer)
            .context("Failed to write settings")?;
    }

    Ok(())
}

fn parse_settings<'a>(args: &'a [&'a str]) -> anyhow::Result<(Settings, &'a [&'a str])> {
    let mut remaining_args = &args[1..];

    #[derive(Default, Debug)]
    struct SettingsBuilder {
        explicit_config_file: bool,
        config_file: Option<PathBuf>,
        db_dir: Option<PathBuf>,
        resolution_minutes: Option<String>,
        write_settings: bool,
        debug: bool,
    }

    let mut b = SettingsBuilder {
        write_settings: true,
        ..Default::default()
    };
    loop {
        match remaining_args {
            ["-C" | "--config-file", config_file, rest @ ..] => {
                b.explicit_config_file = true;
                b.config_file = Some(PathBuf::from(config_file));
                remaining_args = rest;
            }
            ["-R" | "--resolution", resolution, rest @ ..] => {
                b.resolution_minutes = Some(resolution.to_string());
                remaining_args = rest;
            }
            ["-D" | "--db-dir", db_dir, rest @ ..] => {
                b.db_dir = Some(PathBuf::from(db_dir));
                remaining_args = rest;
            }
            ["-W" | "--no_write-settings", rest @ ..] => {
                b.write_settings = false;
                remaining_args = rest;
            }
            ["--debug", rest @ ..] => {
                b.debug = true;
                remaining_args = rest;
            }
            _ => {
                break;
            }
        }
    }

    b.config_file = Some(settings_location(b.config_file)?);

    let from_file = if let Some(ref file) = b.config_file {
        let exists = file.is_file();
        if b.explicit_config_file && !b.write_settings && !exists {
            bail!(
                "Settings file {} does not exist and is not configured to be written",
                file.display()
            );
        }
        if exists {
            let file = std::fs::File::open(file).context("Failed to open settings file")?;
            let reader = BufReader::new(file);
            let explicit: SettingsSer =
                serde_json::from_reader(reader).context("Failed to read settings")?;
            Some(explicit)
        } else {
            None
        }
    } else {
        None
    };

    let db_dir = db_location(b.db_dir, from_file.as_ref())?;
    let resolution = resolution(b.resolution_minutes, from_file.as_ref())?;
    let mut settings = Settings::from_ser(from_file);
    settings.db_dir = db_dir;
    settings.write_settings = b.write_settings;
    settings.resolution = resolution;
    settings.settings_location = b.config_file;
    settings.debug = b.debug;

    Ok((settings, remaining_args))
}

const SETTINGS_FILE_NAME: &str = "quarble_settings.json";

fn settings_location(explicit: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = explicit {
        Ok(explicit)
    } else if let Ok(quarble_home_env) = std::env::var("QUARBLE_HOME") {
        let quarble_home = PathBuf::from(&quarble_home_env);
        if quarble_home.is_absolute() || quarble_home.exists() {
            Ok(quarble_home.join(SETTINGS_FILE_NAME))
        } else {
            bail!(
                "Invalid environment value for 'QUARBLE_HOME': '{}'",
                quarble_home_env
            );
        }
    } else if let Some(data_dir) = dirs::data_dir() {
        Ok(data_dir.join("quarble").join(SETTINGS_FILE_NAME))
    } else {
        bail!("DB location not defined")
    }
}

fn db_location(explicit: Option<PathBuf>, loaded: Option<&SettingsSer>) -> anyhow::Result<PathBuf> {
    if let Some(explicit) = explicit {
        Ok(explicit)
    } else if let Some(SettingsSer { db_dir, .. }) = loaded {
        Ok(db_dir.to_owned())
    } else if let Ok(quarble_home_env) = std::env::var("QUARBLE_HOME") {
        let quarble_home = PathBuf::from(&quarble_home_env);
        if quarble_home.is_absolute() || quarble_home.exists() {
            Ok(quarble_home.join("db"))
        } else {
            bail!(
                "Invalid environment value for 'QUARBLE_HOME': '{}'",
                quarble_home_env
            );
        }
    } else if let Some(data_dir) = dirs::data_dir() {
        Ok(data_dir.join("quarble").join("db"))
    } else {
        bail!("DB location not defined")
    }
}

fn resolution(
    explicit: Option<String>,
    loaded: Option<&SettingsSer>,
) -> anyhow::Result<chrono::Duration> {
    if let Some(explicit) = explicit {
        let minutes =
            u32::from_str(&explicit).context("Cannot parse explicitly provided duration")?;
        Ok(chrono::Duration::minutes(minutes as i64))
    } else if let Some(SettingsSer {
        resolution_minutes, ..
    }) = loaded
    {
        if *resolution_minutes < 1 || *resolution_minutes > 60 {
            bail!(
                "Invalid resolution_minutes in settings file: {}",
                resolution_minutes
            );
        }
        Ok(chrono::Duration::minutes(*resolution_minutes as i64))
    } else {
        Ok(chrono::Duration::minutes(15))
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use chrono::Duration;

    use crate::parse_settings;

    #[test]
    fn parse_args() {
        let input = vec![
            "program_name",
            "--db-dir",
            "explicit-dir",
            "--resolution",
            "5",
        ];

        let (settings, remainder) = parse_settings(&input).unwrap();

        assert!(remainder.is_empty(), "Expected empty: {:?}", remainder);
        assert_eq!(settings.resolution, Duration::minutes(5));
        assert_eq!(settings.db_dir, PathBuf::from("explicit-dir"))
    }
}
