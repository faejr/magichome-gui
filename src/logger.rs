use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use std::{collections::HashMap, sync::Mutex};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref LOGS: Mutex<Vec<String>> = Mutex::new(vec![]);
}

#[macro_export]
macro_rules! log_text {
    () => {
        crate::logger::LOGS.lock().unwrap().join("\n")
    };
}

pub struct VecLogger {
    /// The default logging level
    default_level: LevelFilter,

    /// The specific logging level for each module
    ///
    /// This is used to override the default value for some specific modules.
    /// After initialization, the vector is sorted so that the first (prefix) match
    /// directly gives us the desired log level.
    module_levels: Vec<(String, LevelFilter)>
}

impl VecLogger {
    /// Initializes the global logger with a VecLogger instance with
    /// default log level set to `Level::Trace`.
    ///
    /// ```no_run
    /// use simple_logger::VecLogger;
    /// VecLogger::new().env().init().unwrap();
    /// log::warn!("This is an example message.");
    /// ```
    ///
    /// [`init`]: #method.init
    #[must_use = "You must call init() to begin logging"]
    pub fn new() -> VecLogger {
        VecLogger {
            default_level: LevelFilter::Trace,
            module_levels: Vec::new()
        }
    }

    /// Simulates env_logger behavior, which enables the user to choose log level by
    /// setting a `RUST_LOG` environment variable. The `RUST_LOG` is not set or its value is not
    /// recognized as one of the log levels, this function will use the `Error` level by default.
    ///
    /// You may use the various builder-style methods on this type to configure
    /// the logger, and you must call [`init`] in order to start logging messages.
    ///
    /// ```no_run
    /// use simple_logger::VecLogger;
    /// VecLogger::from_env().init().unwrap();
    /// log::warn!("This is an example message.");
    /// ```
    ///
    /// [`init`]: #method.init
    #[must_use = "You must call init() to begin logging"]
    #[deprecated(
        since = "1.12.0",
        note = "Use [`env`](#method.env) instead. Will be removed in version 2.0.0."
    )]
    pub fn from_env() -> VecLogger {
        VecLogger::new()
            .with_level(log::LevelFilter::Error)
            .env()
    }

    /// Simulates env_logger behavior, which enables the user to choose log
    /// level by setting a `RUST_LOG` environment variable. This will use
    /// the default level set by [`with_level`] if `RUST_LOG` is not set or
    /// can't be parsed as a standard log level.
    ///
    /// [`with_level`]: #method.with_level
    #[must_use = "You must call init() to begin logging"]
    pub fn env(mut self) -> VecLogger {
        if let Ok(level) = std::env::var("RUST_LOG") {
            match level.to_lowercase().as_str() {
                "trace" => self.default_level = log::LevelFilter::Trace,
                "debug" => self.default_level = log::LevelFilter::Debug,
                "info" => self.default_level = log::LevelFilter::Info,
                "warn" => self.default_level = log::LevelFilter::Warn,
                "error" => self.default_level = log::LevelFilter::Error,
                _ => (),
            }
        };
        self
    }

    /// Set the 'default' log level.
    ///
    /// You can override the default level for specific modules and their sub-modules using [`with_module_level`]
    ///
    /// [`with_module_level`]: #method.with_module_level
    #[must_use = "You must call init() to begin logging"]
    pub fn with_level(mut self, level: LevelFilter) -> VecLogger {
        self.default_level = level;
        self
    }

    /// Override the log level for some specific modules.
    ///
    /// This sets the log level of a specific module and all its sub-modules.
    /// When both the level for a parent module as well as a child module are set,
    /// the more specific value is taken. If the log level for the same module is
    /// specified twice, the resulting log level is implementation defined.
    ///
    /// # Examples
    ///
    /// Silence an overly verbose crate:
    ///
    /// ```no_run
    /// use simple_logger::VecLogger;
    /// use log::LevelFilter;
    ///
    /// VecLogger::new().with_module_level("chatty_dependency", LevelFilter::Warn).init().unwrap();
    /// ```
    ///
    /// Disable logging for all dependencies:
    ///
    /// ```no_run
    /// use simple_logger::VecLogger;
    /// use log::LevelFilter;
    ///
    /// VecLogger::new()
    ///     .with_level(LevelFilter::Off)
    ///     .with_module_level("my_crate", LevelFilter::Info)
    ///     .init()
    ///     .unwrap();
    /// ```
    #[must_use = "You must call init() to begin logging"]
    pub fn with_module_level(mut self, target: &str, level: LevelFilter) -> VecLogger {
        self.module_levels.push((target.to_string(), level));

        /* Normally this is only called in `init` to avoid redundancy, but we can't initialize the logger in tests */
        #[cfg(test)]
        self.module_levels
            .sort_by_key(|(name, _level)| name.len().wrapping_neg());

        self
    }

    /// Override the log level for specific targets.
    #[must_use = "You must call init() to begin logging"]
    #[deprecated(
        since = "1.11.0",
        note = "Use [`with_module_level`](#method.with_module_level) instead. Will be removed in version 2.0.0."
    )]
    pub fn with_target_levels(
        mut self,
        target_levels: HashMap<String, LevelFilter>,
    ) -> VecLogger {
        self.module_levels = target_levels.into_iter().collect();

        /* Normally this is only called in `init` to avoid redundancy, but we can't initialize the logger in tests */
        #[cfg(test)]
        self.module_levels
            .sort_by_key(|(name, _level)| name.len().wrapping_neg());

        self
    }
    
    /// 'Init' the actual logger, instantiate it and configure it,
    /// this method MUST be called in order for the logger to be effective.
    pub fn init(mut self) -> Result<(), SetLoggerError> {
        #[cfg(all(windows, feature = "colored"))]
        set_up_color_terminal();

        /* Sort all module levels from most specific to least specific. The length of the module
         * name is used instead of its actual depth to avoid module name parsing.
         */
        self.module_levels
            .sort_by_key(|(name, _level)| name.len().wrapping_neg());
        let max_level = self
            .module_levels
            .iter()
            .map(|(_name, level)| level)
            .copied()
            .max();
        let max_level = max_level
            .map(|lvl| lvl.max(self.default_level))
            .unwrap_or(self.default_level);
        log::set_max_level(max_level);
        log::set_boxed_logger(Box::new(self))?;
        Ok(())
    }
}

impl Default for VecLogger {
    /// See [this](struct.VecLogger.html#method.new)
    fn default() -> Self {
        VecLogger::new()
    }
}

impl Log for VecLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        &metadata.level().to_level_filter()
            <= self
                .module_levels
                .iter()
                /* At this point the Vec is already sorted so that we can simply take
                 * the first match
                 */
                .find(|(name, _level)| metadata.target().starts_with(name))
                .map(|(_name, level)| level)
                .unwrap_or(&self.default_level)
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let level_string = record.level().to_string();

            let target = if !record.target().is_empty() {
                record.target()
            } else {
                record.module_path().unwrap_or_default()
            };

            LOGS.lock().unwrap().push(format!("{:<5} [{}] {}", level_string, target, record.args()));
        }
    }

    fn flush(&self) {}
}
