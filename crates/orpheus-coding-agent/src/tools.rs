use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{bail, Context};
use orpheus::prelude::{Param, Tool};
use orpheus_agent::AgentTool;
use serde::Deserialize;
use wait_timeout::ChildExt;

const MAX_READ_LINES: usize = 2000;
const MAX_READ_BYTES: usize = 50 * 1024;

#[derive(Debug, Clone)]
pub struct BuiltinToolset {
    tools: Vec<AgentTool>,
}

impl BuiltinToolset {
    pub fn new(cwd: PathBuf, disabled_tools: &[String]) -> Self {
        let is_disabled = |name: &str| disabled_tools.iter().any(|item| item == name);
        let mut tools = Vec::new();

        if !is_disabled("read") {
            tools.push(read_tool(cwd.clone()));
        }
        if !is_disabled("write") {
            tools.push(write_tool(cwd.clone()));
        }
        if !is_disabled("edit") {
            tools.push(edit_tool(cwd.clone()));
        }
        if !is_disabled("bash") {
            tools.push(bash_tool(cwd));
        }

        Self { tools }
    }

    pub fn names() -> &'static [&'static str] {
        &["read", "write", "edit", "bash"]
    }

    pub fn into_tools(self) -> Vec<AgentTool> {
        self.tools
    }
}

#[derive(Debug, Deserialize)]
struct ReadArgs {
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct WriteArgs {
    path: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct EditArgs {
    path: String,
    edits: Vec<EditOperation>,
}

#[derive(Debug, Deserialize)]
struct EditOperation {
    #[serde(alias = "oldText")]
    old_text: String,
    #[serde(alias = "newText")]
    new_text: String,
}

#[derive(Debug, Deserialize)]
struct BashArgs {
    command: String,
    timeout: Option<u64>,
}

fn read_tool(cwd: PathBuf) -> AgentTool {
    AgentTool::new(
        Tool::function("read")
            .description("Read a UTF-8 text file from disk. Supports optional line-based offset and limit.")
            .with_parameters(|params| {
                params
                    .property("path", Param::string().description("Path to the file to read"))
                    .property(
                        "offset",
                        Param::integer().description("1-indexed line number to start reading from"),
                    )
                    .property(
                        "limit",
                        Param::integer().description("Maximum number of lines to read"),
                    )
                    .required(["path"])
            })
            .build(),
        move |call| {
            let args: ReadArgs = serde_json::from_str(&call.arguments)
                .context("failed to parse read tool arguments")?;
            let path = resolve_path(&cwd, &args.path);
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed to read UTF-8 file: {}", path.display()))?;
            Ok(read_window(&content, args.offset, args.limit))
        },
    )
}

fn write_tool(cwd: PathBuf) -> AgentTool {
    AgentTool::new(
        Tool::function("write")
            .description("Create or overwrite a text file.")
            .with_parameters(|params| {
                params
                    .property("path", Param::string().description("Path to the file to write"))
                    .property("content", Param::string().description("Full file contents"))
                    .required(["path", "content"])
            })
            .build(),
        move |call| {
            let args: WriteArgs = serde_json::from_str(&call.arguments)
                .context("failed to parse write tool arguments")?;
            let path = resolve_path(&cwd, &args.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, args.content.as_bytes())
                .with_context(|| format!("failed to write file: {}", path.display()))?;
            Ok(format!("Wrote {}", path.display()))
        },
    )
}

fn edit_tool(cwd: PathBuf) -> AgentTool {
    AgentTool::new(
        Tool::function("edit")
            .description("Apply exact text replacements to a UTF-8 text file. Each old_text must match exactly once in the original file.")
            .with_parameters(|params| {
                params
                    .property("path", Param::string().description("Path to the file to edit"))
                    .property(
                        "edits",
                        Param::array()
                            .description("List of exact replacements to apply against the original file")
                            .items(
                                Param::object()
                                    .property(
                                        "old_text",
                                        Param::string().description("Exact text to replace; must match uniquely"),
                                    )
                                    .property(
                                        "new_text",
                                        Param::string().description("Replacement text"),
                                    )
                                    .required(["old_text", "new_text"]),
                            ),
                    )
                    .required(["path", "edits"])
            })
            .build(),
        move |call| {
            let args: EditArgs = serde_json::from_str(&call.arguments)
                .context("failed to parse edit tool arguments")?;
            let path = resolve_path(&cwd, &args.path);
            let original = fs::read_to_string(&path)
                .with_context(|| format!("failed to read UTF-8 file: {}", path.display()))?;
            let updated = apply_exact_replacements(&original, &args.edits)?;
            fs::write(&path, updated.as_bytes())
                .with_context(|| format!("failed to write file: {}", path.display()))?;
            Ok(format!("Edited {} with {} replacement(s)", path.display(), args.edits.len()))
        },
    )
}

fn bash_tool(cwd: PathBuf) -> AgentTool {
    AgentTool::new(
        Tool::function("bash")
            .description("Run a shell command in the working directory and return stdout and stderr.")
            .with_parameters(|params| {
                params
                    .property("command", Param::string().description("Shell command to execute"))
                    .property(
                        "timeout",
                        Param::integer().description("Optional timeout in seconds before terminating the command"),
                    )
                    .required(["command"])
            })
            .build(),
        move |call| {
            let args: BashArgs = serde_json::from_str(&call.arguments)
                .context("failed to parse bash tool arguments")?;
            Ok(run_command(&cwd, &args.command, args.timeout)?)
        },
    )
}

fn resolve_path(cwd: &Path, path: &str) -> PathBuf {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        candidate
    } else {
        cwd.join(candidate)
    }
}

fn read_window(content: &str, offset: Option<usize>, limit: Option<usize>) -> String {
    let start = offset.unwrap_or(1).max(1) - 1;
    let limit = limit.unwrap_or(MAX_READ_LINES).max(1).min(MAX_READ_LINES);

    let mut out = String::new();
    let mut lines_written = 0usize;

    for line in content.lines().skip(start).take(limit) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line);
        lines_written += 1;

        if out.len() >= MAX_READ_BYTES {
            out.truncate(MAX_READ_BYTES.min(out.len()));
            out.push_str("\n[truncated]");
            return out;
        }
    }

    if lines_written == 0 {
        String::from("[no content]")
    } else {
        out
    }
}

fn apply_exact_replacements(original: &str, edits: &[EditOperation]) -> anyhow::Result<String> {
    if edits.is_empty() {
        return Ok(original.to_string());
    }

    let mut matches = Vec::with_capacity(edits.len());

    for edit in edits {
        let (start, end) = unique_match(original, &edit.old_text)?;
        matches.push((start, end, edit));
    }

    matches.sort_by_key(|(start, _, _)| *start);

    for pair in matches.windows(2) {
        let (_, end_a, _) = pair[0];
        let (start_b, _, _) = pair[1];
        if start_b < end_a {
            bail!("edit ranges overlap in the original file")
        }
    }

    let mut result = String::with_capacity(original.len());
    let mut cursor = 0usize;

    for (start, end, edit) in matches {
        result.push_str(&original[cursor..start]);
        result.push_str(&edit.new_text);
        cursor = end;
    }

    result.push_str(&original[cursor..]);
    Ok(result)
}

fn unique_match(haystack: &str, needle: &str) -> anyhow::Result<(usize, usize)> {
    if needle.is_empty() {
        bail!("edit old_text cannot be empty")
    }

    let mut positions = haystack.match_indices(needle);
    let Some((start, matched)) = positions.next() else {
        bail!("edit old_text did not match the file")
    };

    if positions.next().is_some() {
        bail!("edit old_text matched more than once; refine the replacement target")
    }

    Ok((start, start + matched.len()))
}

fn run_command(cwd: &Path, command: &str, timeout_secs: Option<u64>) -> anyhow::Result<String> {
    let mut child = shell_command(command)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to spawn shell command in {}", cwd.display()))?;

    let status = if let Some(timeout_secs) = timeout_secs {
        let timeout = Duration::from_secs(timeout_secs.max(1));
        match child.wait_timeout(timeout)? {
            Some(status) => status,
            None => {
                child.kill()?;
                let _ = child.wait();
                return Ok(format!("Command timed out after {}s", timeout_secs));
            }
        }
    } else {
        child.wait()?
    };

    let mut stdout = String::new();
    if let Some(mut handle) = child.stdout.take() {
        handle.read_to_string(&mut stdout)?;
    }

    let mut stderr = String::new();
    if let Some(mut handle) = child.stderr.take() {
        handle.read_to_string(&mut stderr)?;
    }

    let mut output = String::new();
    output.push_str(&format!("exit_status: {}\n", status));

    if !stdout.is_empty() {
        output.push_str("stdout:\n");
        output.push_str(&truncate_output(&stdout));
        output.push('\n');
    }

    if !stderr.is_empty() {
        output.push_str("stderr:\n");
        output.push_str(&truncate_output(&stderr));
        output.push('\n');
    }

    if stdout.is_empty() && stderr.is_empty() {
        output.push_str("[no output]");
    }

    Ok(output)
}

fn truncate_output(output: &str) -> String {
    if output.len() <= MAX_READ_BYTES {
        output.to_string()
    } else {
        let mut truncated = output[..MAX_READ_BYTES].to_string();
        truncated.push_str("\n[truncated]");
        truncated
    }
}

#[cfg(unix)]
fn shell_command(command: &str) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-lc").arg(command);
    cmd
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg(command);
    cmd
}

#[cfg(test)]
mod tests {
    use super::{EditOperation, apply_exact_replacements};

    #[test]
    fn applies_multiple_non_overlapping_edits_against_original() {
        let original = "alpha beta gamma";
        let edits = vec![
            EditOperation {
                old_text: String::from("alpha"),
                new_text: String::from("one"),
            },
            EditOperation {
                old_text: String::from("gamma"),
                new_text: String::from("three"),
            },
        ];

        let updated = apply_exact_replacements(original, &edits).expect("edits apply");
        assert_eq!(updated, "one beta three");
    }

    #[test]
    fn rejects_non_unique_match() {
        let original = "dup dup";
        let edits = vec![EditOperation {
            old_text: String::from("dup"),
            new_text: String::from("value"),
        }];

        let error = apply_exact_replacements(original, &edits).expect_err("edit should fail");
        assert!(error.to_string().contains("matched more than once"));
    }
}
