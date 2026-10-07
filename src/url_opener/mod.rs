mod error;
mod runner;
mod system;
mod tool;

pub use error::UrlOpenError;
pub use tool::BrowserChoice;

use runner::UrlOpenCommandRunner;
use system::SystemCommandRunner;
use tool::{UrlOpenEnvironment, UrlOpenTool};

/**
 * Successful URL launch metadata.
 */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenUrlSuccess {
    pub tool: String,
}

/**
 * Opens URLs in a browser.
 */
pub trait UrlOpener {
    fn open(&self, url: &str) -> Result<OpenUrlSuccess, UrlOpenError>;
}

/**
 * Production URL opener launching the configured browser.
 */
#[derive(Debug, Default, Clone)]
pub struct SystemUrlOpener {
    browser: BrowserChoice,
}

impl SystemUrlOpener {
    pub fn new(browser: BrowserChoice) -> Self {
        Self { browser }
    }
}

impl UrlOpener for SystemUrlOpener {
    fn open(&self, url: &str) -> Result<OpenUrlSuccess, UrlOpenError> {
        open_with_runner(
            url,
            &self.browser,
            &SystemCommandRunner,
            UrlOpenEnvironment::current(),
        )
    }
}

/// Tries each candidate launcher in order, returning the first success or the last error.
fn open_with_runner(
    url: &str,
    browser: &BrowserChoice,
    runner: &impl UrlOpenCommandRunner,
    env: UrlOpenEnvironment,
) -> Result<OpenUrlSuccess, UrlOpenError> {
    let mut last_error = UrlOpenError::UnsupportedPlatform;
    for tool in UrlOpenTool::candidates(browser, env)? {
        if !runner.command_exists(&tool.program) {
            last_error = UrlOpenError::NoToolFound {
                tool: tool.program.clone(),
            };
            continue;
        }
        match runner.run(&tool, url) {
            Ok(()) => return Ok(OpenUrlSuccess { tool: tool.label() }),
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// Fake runner where `available` commands exist and `failing` labels exit non-zero.
    #[derive(Default)]
    struct FakeRunner {
        available: Vec<&'static str>,
        failing: Vec<&'static str>,
        runs: RefCell<Vec<(String, String)>>,
    }

    impl UrlOpenCommandRunner for FakeRunner {
        fn command_exists(&self, command: &str) -> bool {
            self.available.contains(&command)
        }

        fn run(&self, tool: &UrlOpenTool, url: &str) -> Result<(), UrlOpenError> {
            self.runs.borrow_mut().push((tool.label(), url.to_string()));
            if self.failing.contains(&tool.label().as_str()) {
                Err(UrlOpenError::CommandFailed {
                    tool: tool.label(),
                    status: "exit status: 1".into(),
                })
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn macos_default_opens_chrome_with_the_url_as_one_argument() {
        let runner = FakeRunner {
            available: vec!["open"],
            ..FakeRunner::default()
        };
        let success = open_with_runner(
            "https://example.com/a b",
            &BrowserChoice::PreferChrome,
            &runner,
            UrlOpenEnvironment::Macos,
        )
        .unwrap();

        assert_eq!(success.tool, "open -a Google Chrome");
        assert_eq!(
            runner.runs.borrow().as_slice(),
            [(
                "open -a Google Chrome".to_string(),
                "https://example.com/a b".to_string()
            )]
        );
    }

    #[test]
    fn default_falls_back_to_system_handler_when_chrome_fails() {
        let runner = FakeRunner {
            available: vec!["open"],
            failing: vec!["open -a Google Chrome"],
            ..FakeRunner::default()
        };
        let success = open_with_runner(
            "https://example.com",
            &BrowserChoice::PreferChrome,
            &runner,
            UrlOpenEnvironment::Macos,
        )
        .unwrap();

        assert_eq!(success.tool, "open");
        assert_eq!(runner.runs.borrow().len(), 2);
    }

    #[test]
    fn linux_default_skips_missing_chrome_for_xdg_open() {
        let runner = FakeRunner {
            available: vec!["xdg-open"],
            ..FakeRunner::default()
        };
        let success = open_with_runner(
            "file:///tmp/test",
            &BrowserChoice::PreferChrome,
            &runner,
            UrlOpenEnvironment::Linux,
        )
        .unwrap();

        assert_eq!(success.tool, "xdg-open");
    }

    #[test]
    fn explicit_browser_failure_is_reported_without_fallback() {
        let runner = FakeRunner {
            available: vec!["open"],
            failing: vec!["open -a Firefox"],
            ..FakeRunner::default()
        };
        let error = open_with_runner(
            "https://example.com",
            &BrowserChoice::Named("firefox".into()),
            &runner,
            UrlOpenEnvironment::Macos,
        )
        .unwrap_err();

        assert!(matches!(error, UrlOpenError::CommandFailed { .. }));
        assert_eq!(runner.runs.borrow().len(), 1);
    }

    #[test]
    fn reports_missing_and_unsupported_launchers() {
        let error = open_with_runner(
            "https://example.com",
            &BrowserChoice::SystemDefault,
            &FakeRunner::default(),
            UrlOpenEnvironment::Linux,
        )
        .unwrap_err();
        assert_eq!(
            error,
            UrlOpenError::NoToolFound {
                tool: "xdg-open".into()
            }
        );

        let error = open_with_runner(
            "https://example.com",
            &BrowserChoice::SystemDefault,
            &FakeRunner::default(),
            UrlOpenEnvironment::Unsupported,
        )
        .unwrap_err();
        assert_eq!(error, UrlOpenError::UnsupportedPlatform);
    }
}
