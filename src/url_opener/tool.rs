use super::error::UrlOpenError;
use serde::{Deserialize, Serialize};

/// Which program opens URLs selected by the open-url action.
///
/// Resolved from the global config before the picker pane launches and carried
/// in the picker snapshot.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowserChoice {
    /// Google Chrome, falling back to the system default handler if Chrome
    /// can't be launched. Used when no browser is configured.
    #[default]
    PreferChrome,
    /// The platform default handler (`open` / `xdg-open`).
    SystemDefault,
    /// A browser alias (`chrome`, `firefox`, ...), a macOS application name,
    /// or a Linux command. No fallback if it fails.
    Named(String),
    /// Explicit argv; the URL is appended as the final argument.
    Command(Vec<String>),
}

/// One concrete launch command: `program args... <url>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UrlOpenTool {
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
}

/// Operating-system family used to choose the URL launcher.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UrlOpenEnvironment {
    Macos,
    Linux,
    Unsupported,
}

impl UrlOpenEnvironment {
    pub(crate) fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Unsupported
        }
    }
}

/// Browser aliases mapped to (macOS application name, Linux command).
const BROWSER_ALIASES: &[(&str, &str, &str)] = &[
    ("chrome", "Google Chrome", "google-chrome"),
    ("google-chrome", "Google Chrome", "google-chrome"),
    ("chromium", "Chromium", "chromium"),
    ("firefox", "Firefox", "firefox"),
    ("safari", "Safari", "safari"),
    ("arc", "Arc", "arc"),
    ("brave", "Brave Browser", "brave-browser"),
    ("edge", "Microsoft Edge", "microsoft-edge"),
];

impl UrlOpenTool {
    fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(|arg| arg.to_string()).collect(),
        }
    }

    /// Returns launch commands to try in order; later entries are fallbacks.
    pub(crate) fn candidates(
        choice: &BrowserChoice,
        environment: UrlOpenEnvironment,
    ) -> Result<Vec<Self>, UrlOpenError> {
        match choice {
            BrowserChoice::Command(argv) => {
                let (program, args) = argv.split_first().ok_or(UrlOpenError::EmptyCommand)?;
                Ok(vec![Self {
                    program: program.clone(),
                    args: args.to_vec(),
                }])
            }
            _ if environment == UrlOpenEnvironment::Unsupported => {
                Err(UrlOpenError::UnsupportedPlatform)
            }
            BrowserChoice::PreferChrome => Ok(vec![
                Self::named("chrome", environment),
                Self::system_default(environment),
            ]),
            BrowserChoice::SystemDefault => Ok(vec![Self::system_default(environment)]),
            BrowserChoice::Named(name) => Ok(vec![Self::named(name, environment)]),
        }
    }

    fn system_default(environment: UrlOpenEnvironment) -> Self {
        match environment {
            UrlOpenEnvironment::Linux => Self::new("xdg-open", &[]),
            _ => Self::new("open", &[]),
        }
    }

    /// macOS opens the application via `open -a`; Linux runs the command directly.
    fn named(name: &str, environment: UrlOpenEnvironment) -> Self {
        let alias = BROWSER_ALIASES
            .iter()
            .find(|(alias, _, _)| alias.eq_ignore_ascii_case(name.trim()));
        match environment {
            UrlOpenEnvironment::Linux => {
                Self::new(alias.map_or(name, |(_, _, command)| command), &[])
            }
            _ => Self::new("open", &["-a", alias.map_or(name, |(_, app, _)| app)]),
        }
    }

    /// Human-readable command name for logs and error messages.
    pub(crate) fn label(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels(choice: BrowserChoice, environment: UrlOpenEnvironment) -> Vec<String> {
        UrlOpenTool::candidates(&choice, environment)
            .unwrap()
            .iter()
            .map(UrlOpenTool::label)
            .collect()
    }

    #[test]
    fn default_prefers_chrome_then_system_handler() {
        assert_eq!(
            labels(BrowserChoice::PreferChrome, UrlOpenEnvironment::Macos),
            ["open -a Google Chrome", "open"]
        );
        assert_eq!(
            labels(BrowserChoice::PreferChrome, UrlOpenEnvironment::Linux),
            ["google-chrome", "xdg-open"]
        );
    }

    #[test]
    fn named_browser_resolves_aliases_case_insensitively_without_fallback() {
        assert_eq!(
            labels(
                BrowserChoice::Named("Firefox".into()),
                UrlOpenEnvironment::Macos
            ),
            ["open -a Firefox"]
        );
        assert_eq!(
            labels(
                BrowserChoice::Named("brave".into()),
                UrlOpenEnvironment::Linux
            ),
            ["brave-browser"]
        );
    }

    #[test]
    fn unknown_name_is_used_verbatim() {
        assert_eq!(
            labels(
                BrowserChoice::Named("Orion".into()),
                UrlOpenEnvironment::Macos
            ),
            ["open -a Orion"]
        );
    }

    #[test]
    fn command_is_used_as_argv() {
        assert_eq!(
            labels(
                BrowserChoice::Command(vec!["chrome".into(), "--incognito".into()]),
                UrlOpenEnvironment::Unsupported
            ),
            ["chrome --incognito"]
        );
        assert_eq!(
            UrlOpenTool::candidates(
                &BrowserChoice::Command(Vec::new()),
                UrlOpenEnvironment::Macos
            ),
            Err(UrlOpenError::EmptyCommand)
        );
    }
}
