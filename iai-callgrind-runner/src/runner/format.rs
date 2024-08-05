use std::fmt::{Display, Write};

use anyhow::Result;
use colored::{ColoredString, Colorize};

use super::args::NoCapture;
use super::common::ModulePath;
use super::meta::Metadata;
use super::summary::{CostsDiff, CostsSummary};
use super::tool::ValgrindTool;
use crate::api::{self, EventKind};
use crate::util::{to_string_signed_short, truncate_str_utf8};

pub struct ComparisonHeader {
    pub function_name: String,
    pub id: String,
    pub details: Option<String>,
}

pub struct Header {
    pub module_path: String,
    pub id: Option<String>,
    pub description: Option<String>,
}

pub trait Formatter {
    fn format_float(float: f64, unit: &str) -> ColoredString {
        let signed_short = to_string_signed_short(float);
        if float.is_infinite() {
            if float.is_sign_positive() {
                format!("{signed_short:+^9}").bright_red().bold()
            } else {
                format!("{signed_short:+^9}").bright_green().bold()
            }
        } else if float.is_sign_positive() {
            format!("{signed_short:^+8}{unit}").bright_red().bold()
        } else {
            format!("{signed_short:^+8}{unit}").bright_green().bold()
        }
    }

    fn format(
        &self,
        baselines: (Option<String>, Option<String>),
        costs_summary: &CostsSummary,
    ) -> Result<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputFormat {
    Default,
    Json,
    PrettyJson,
}

#[derive(Clone)]
pub struct VerticalFormat {
    event_kinds: Vec<EventKind>,
}

impl ComparisonHeader {
    pub fn new<T, U, V>(function_name: T, id: U, details: Option<V>) -> Self
    where
        T: Into<String>,
        U: Into<String>,
        V: Into<String>,
    {
        Self {
            function_name: function_name.into(),
            id: id.into(),
            details: details.map(Into::into),
        }
    }

    pub fn print(&self) {
        println!("{self}");
    }
}

impl Display for ComparisonHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  {} {} {}",
            "Comparison with".yellow().bold(),
            self.function_name.green(),
            self.id.cyan()
        )?;

        if let Some(details) = &self.details {
            write!(f, ":{}", details.blue().bold())?;
        }

        Ok(())
    }
}

impl Header {
    pub fn new<T, U, V>(module_path: T, id: U, description: V) -> Self
    where
        T: Into<String>,
        U: Into<Option<String>>,
        V: Into<Option<String>>,
    {
        Self {
            module_path: module_path.into(),
            id: id.into(),
            description: description.into(),
        }
    }

    // pub fn from_segments<I, T, U, V>(module_path: T, id: U, description: V) -> Self
    // where
    //     I: AsRef<str>,
    //     T: AsRef<[I]>,
    //     U: Into<Option<String>>,
    //     V: Into<Option<String>>,
    // {
    //     Self {
    //         module_path: module_path
    //             .as_ref()
    //             .iter()
    //             .map(|s| s.as_ref().to_owned())
    //             .collect::<Vec<String>>()
    //             .join("::"),
    //         id: id.into(),
    //         description: description.into(),
    //     }
    // }

    pub fn from_module_path<U, V>(module_path: &ModulePath, id: U, description: V) -> Self
    where
        U: Into<Option<String>>,
        V: Into<Option<String>>,
    {
        Self {
            module_path: module_path.to_string(),
            id: id.into(),
            description: description.into(),
        }
    }

    pub fn print(&self) {
        println!("{self}");
    }

    pub fn to_title(&self) -> String {
        let mut output = String::new();
        write!(&mut output, "{}", self.module_path).unwrap();
        if let Some(id) = &self.id {
            if let Some(description) = &self.description {
                let truncated = truncate_str_utf8(description, 37);
                write!(
                    &mut output,
                    " {id}:{truncated}{}",
                    if truncated.len() < description.len() {
                        "..."
                    } else {
                        ""
                    }
                )
                .unwrap();
            } else {
                write!(&mut output, " {id}").unwrap();
            }
        }
        output
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.module_path.green()))?;
        if let Some(id) = &self.id {
            if let Some(description) = &self.description {
                let truncated = truncate_str_utf8(description, 37);
                f.write_fmt(format_args!(
                    " {}{}{}{}",
                    id.cyan(),
                    ":".cyan(),
                    truncated.bold().blue(),
                    if truncated.len() < description.len() {
                        "..."
                    } else {
                        ""
                    }
                ))?;
            } else {
                f.write_fmt(format_args!(" {}", id.cyan()))?;
            }
        } else if let Some(description) = &self.description {
            let truncated = truncate_str_utf8(description, 37);
            f.write_fmt(format_args!(
                " {}{}",
                truncated.bold().blue(),
                if truncated.len() < description.len() {
                    "..."
                } else {
                    ""
                }
            ))?;
        } else {
            // do nothing
        }
        Ok(())
    }
}

impl VerticalFormat {
    pub fn print(
        &self,
        meta: &Metadata,
        baselines: (Option<String>, Option<String>),
        costs_summary: &CostsSummary,
    ) -> Result<()> {
        if meta.args.output_format == OutputFormat::Default {
            print!("{}", self.format(baselines, costs_summary)?);
        }
        Ok(())
    }
}

impl Default for VerticalFormat {
    fn default() -> Self {
        use EventKind::*;
        Self {
            event_kinds: vec![
                Ir,
                L1hits,
                LLhits,
                RamHits,
                TotalRW,
                EstimatedCycles,
                SysCount,
                SysTime,
                SysCpuTime,
                Ge,
                Bc,
                Bcm,
                Bi,
                Bim,
                ILdmr,
                DLdmr,
                DLdmw,
                AcCost1,
                AcCost2,
                SpLoss1,
                SpLoss2,
            ],
        }
    }
}

impl Formatter for VerticalFormat {
    fn format(
        &self,
        baselines: (Option<String>, Option<String>),
        costs_summary: &CostsSummary,
    ) -> Result<String> {
        format_vertical(
            baselines,
            self.event_kinds
                .iter()
                .filter_map(|e| costs_summary.diff_by_kind(e).map(|d| (e, d))),
        )
    }
}

pub const NOT_AVAILABLE: &str = "N/A";

pub fn format_vertical<'a, K: Display + 'a>(
    baselines: (Option<String>, Option<String>),
    costs_summary: impl Iterator<Item = (&'a K, &'a CostsDiff)>,
) -> Result<String> {
    let mut result = String::new();

    let unknown = "*********";
    let no_change = "No change";

    match baselines {
        (None, None) => {}
        (None, Some(base)) => {
            writeln!(result, "  {:<33}|{base}", "Baselines:").unwrap();
        }
        (Some(base), None) => {
            writeln!(result, "  {:<18}{:>15}", "Baselines:", base.bold()).unwrap();
        }
        (Some(new), Some(old)) => {
            writeln!(result, "  {:<18}{:>15}|{old}", "Baselines:", new.bold()).unwrap();
        }
    }

    for (event_kind, diff) in costs_summary {
        let description = format!("{event_kind}:");
        match (diff.new, diff.old) {
            (None, Some(old_cost)) => writeln!(
                result,
                "  {description:<18}{:>15}|{old_cost:<15} ({:^9})",
                NOT_AVAILABLE.bold(),
                unknown.bright_black()
            )?,
            (Some(new_cost), None) => writeln!(
                result,
                "  {description:<18}{:>15}|{NOT_AVAILABLE:<15} ({:^9})",
                new_cost.to_string().bold(),
                unknown.bright_black()
            )?,
            (Some(new_cost), Some(old_cost)) if new_cost == old_cost => writeln!(
                result,
                "  {description:<18}{:>15}|{old_cost:<15} ({:^9})",
                new_cost.to_string().bold(),
                no_change.bright_black()
            )?,
            (Some(new_cost), Some(old_cost)) => {
                let pct_string = {
                    let pct = diff.diff_pct.expect(
                        "If there are new costs and old costs there should be a difference in \
                         percent",
                    );
                    VerticalFormat::format_float(pct, "%")
                };
                let factor_string = {
                    let factor = diff.factor.expect(
                        "If there are new costs and old costs there should be a difference factor",
                    );
                    VerticalFormat::format_float(factor, "x")
                };
                writeln!(
                    result,
                    "  {description:<18}{:>15}|{old_cost:<15} ({pct_string:^9}) \
                     [{factor_string:^9}]",
                    new_cost.to_string().bold(),
                )?;
            }
            _ => {}
        }
    }
    Ok(result)
}

pub fn tool_headline(tool: ValgrindTool) -> String {
    let id = tool.id();
    format!(
        "  {} {} {}",
        "=======".bright_black(),
        id.to_ascii_uppercase(),
        "=".repeat(64 - id.len()).bright_black(),
        // "=".repeat(34 - tool.id().len()).bright_black()
    )
}

// Return the formatted `String` if `NoCapture` is not `False`
pub fn no_capture_footer(nocapture: NoCapture) -> Option<String> {
    match nocapture {
        NoCapture::True => Some(format!(
            "{} {}",
            "-".yellow(),
            "end of stdout/stderr".yellow()
        )),
        NoCapture::False => None,
        NoCapture::Stderr => Some(format!("{} {}", "-".yellow(), "end of stderr".yellow())),
        NoCapture::Stdout => Some(format!("{} {}", "-".yellow(), "end of stdout".yellow())),
    }
}

pub fn print_no_capture_footer(
    nocapture: NoCapture,
    stdout: Option<&api::Stdio>,
    stderr: Option<&api::Stdio>,
) {
    let stdout_is_pipe = stdout.map_or(
        nocapture == NoCapture::False || nocapture == NoCapture::Stderr,
        api::Stdio::is_pipe,
    );

    let stderr_is_pipe = stderr.map_or(
        nocapture == NoCapture::False || nocapture == NoCapture::Stdout,
        api::Stdio::is_pipe,
    );

    // These unwraps are safe because `no_capture_footer` returns None only if `NoCapture` is
    // `False`
    match (stdout_is_pipe, stderr_is_pipe) {
        (true, true) => {}
        (true, false) => {
            println!("{}", no_capture_footer(NoCapture::Stderr).unwrap());
        }
        (false, true) => {
            println!("{}", no_capture_footer(NoCapture::Stdout).unwrap());
        }
        (false, false) => {
            println!("{}", no_capture_footer(NoCapture::True).unwrap());
        }
    }
}
