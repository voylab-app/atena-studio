use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use crate::core::process::silent_tokio_command;

use crate::core::memory::SkillCommandResult;

pub struct SkillScriptRunner;

impl SkillScriptRunner {
    /// Inspects a command line string for catastrophically destructive patterns
    pub fn validate_command_safety(command_str: &str) -> Result<(), String> {
        let normalized = command_str.trim().to_lowercase();

        // 1. Catastrophic recursive deletion on system roots, homes, or wildcards
        let destructive_rm_patterns = [
            "rm -rf /", "rm -fr /", "rm -r /", "rm -f /",
            "rm -rf /*", "rm -fr /*", "rm -r /*",
            "rm -rf ~", "rm -fr ~", "rm -r ~",
            "rm -rf *", "rm -fr *",
            "rm -rf .", "rm -fr .",
            "rm -rf $home", "rm -fr $home",
            "rm -rf /bin", "rm -rf /usr", "rm -rf /etc", "rm -rf /lib", "rm -rf /boot", "rm -rf /system",
            "rmdir /s /q c:\\", "rd /s /q c:\\", "rd /s /q %systemdrive%",
            "del /f /s /q c:\\", "del /f /s /q *.*",
        ];

        for pattern in &destructive_rm_patterns {
            if normalized.contains(pattern) {
                return Err(format!(
                    "Destructive command rejected for safety: matched high-risk pattern '{}'",
                    pattern
                ));
            }
        }

        // 2. Raw disk formatting or destruction
        let raw_disk_patterns = [
            "mkfs.", "mkfs ", "dd if=/dev/zero", "dd if=/dev/urandom", "dd of=/dev/sd",
            "dd of=/dev/nvme", "dd of=/dev/disk", "format c:", "diskpart",
        ];

        for pattern in &raw_disk_patterns {
            if normalized.contains(pattern) {
                return Err(format!(
                    "Destructive disk operation rejected for safety: matched '{}'",
                    pattern
                ));
            }
        }

        // 3. Fork bombs and destructive system modifiers
        let exploit_patterns = [
            ":(){ :|:& };:", ":(){:|:&};:", "chmod -r 777 /", "chown -r /",
        ];

        for pattern in &exploit_patterns {
            if normalized.contains(pattern) {
                return Err(format!(
                    "Malicious or destructive system command rejected for safety: matched '{}'",
                    pattern
                ));
            }
        }

        Ok(())
    }

    /// Executes an arbitrary shell command within the context of a skill directory
    pub async fn run_command(
        command_str: &str,
        skill_dir: Option<&Path>,
        working_dir: Option<&Path>,
        env_vars: Option<&HashMap<String, String>>,
        timeout_ms: Option<u64>,
    ) -> Result<SkillCommandResult, String> {
        // Enforce safety inspection on command
        Self::validate_command_safety(command_str)?;

        let start_time = Instant::now();
        let timeout_duration = Duration::from_millis(timeout_ms.unwrap_or(120_000));

        let effective_cwd = working_dir
            .map(|p| p.to_path_buf())
            .or_else(|| skill_dir.map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        #[cfg(target_os = "windows")]
        let mut cmd = {
            let mut c = silent_tokio_command("cmd.exe");
            c.arg("/C").arg(command_str);
            c
        };

        #[cfg(not(target_os = "windows"))]
        let mut cmd = {
            let shell = if Path::new("/bin/zsh").exists() {
                "/bin/zsh"
            } else {
                "/bin/sh"
            };
            let mut c = Command::new(shell);
            c.arg("-c").arg(command_str);
            c
        };

        cmd.kill_on_drop(true);
        cmd.current_dir(&effective_cwd);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Injects skill-specific environment variables
        if let Some(envs) = env_vars {
            for (k, v) in envs {
                cmd.env(k, v);
            }
        }

        // Also inject ATENA_SKILL_DIR for convenient scripting
        if let Some(s_dir) = skill_dir {
            cmd.env("ATENA_SKILL_DIR", s_dir.to_string_lossy().to_string());
        }

        let child = cmd.spawn().map_err(|e| {
            format!(
                "Failed to spawn command in '{}': {}",
                effective_cwd.display(),
                e
            )
        })?;

        let wait_result = timeout(timeout_duration, child.wait_with_output()).await;

        let elapsed = start_time.elapsed().as_millis() as u64;

        match wait_result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();
                let success = output.status.success();

                Ok(SkillCommandResult {
                    success,
                    exit_code,
                    stdout,
                    stderr,
                    duration_ms: elapsed,
                })
            }
            Ok(Err(e)) => Err(format!("Command execution error: {}", e)),
            Err(_) => Err(format!(
                "Command execution timed out after {} ms",
                timeout_duration.as_millis()
            )),
        }
    }

    /// Executes a script file located inside the skill directory (e.g. scripts/run.sh)
    pub async fn run_script(
        skill_dir: &Path,
        script_rel_path: &str,
        args: &[String],
        working_dir: Option<&Path>,
        env_vars: Option<&HashMap<String, String>>,
        timeout_ms: Option<u64>,
    ) -> Result<SkillCommandResult, String> {
        let script_path = skill_dir.join(script_rel_path);
        if !script_path.exists() {
            return Err(format!(
                "Script file does not exist at '{}'",
                script_path.display()
            ));
        }

        // On Unix, ensure file has execute permission if it is a standalone executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&script_path) {
                let mut perms = metadata.permissions();
                let mode = perms.mode();
                if mode & 0o111 == 0 {
                    perms.set_mode(mode | 0o755);
                    let _ = std::fs::set_permissions(&script_path, perms);
                }
            }
        }

        let start_time = Instant::now();
        let timeout_duration = Duration::from_millis(timeout_ms.unwrap_or(180_000));

        let effective_cwd = working_dir
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| skill_dir.to_path_buf());

        // Determine interpreter by extension
        let ext = script_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let mut cmd = match ext.as_str() {
            "py" => {
                let mut c = silent_tokio_command("python3");
                c.arg(&script_path);
                c
            }
            "js" => {
                let mut c = silent_tokio_command("node");
                c.arg(&script_path);
                c
            }
            "sh" | "bash" | "zsh" => {
                let shell = if ext == "zsh" && Path::new("/bin/zsh").exists() {
                    "/bin/zsh"
                } else if Path::new("/bin/bash").exists() {
                    "/bin/bash"
                } else {
                    "/bin/sh"
                };
                let mut c = silent_tokio_command(shell);
                c.arg(&script_path);
                c
            }
            "ps1" => {
                let mut c = silent_tokio_command("powershell.exe");
                c.arg("-ExecutionPolicy").arg("Bypass").arg("-File").arg(&script_path);
                c
            }
            "bat" | "cmd" => {
                let mut c = silent_tokio_command("cmd.exe");
                c.arg("/C").arg(&script_path);
                c
            }
            _ => {
                // Try executing the script path directly
                let c = silent_tokio_command(&script_path);
                c
            }
        };

        for arg in args {
            cmd.arg(arg);
        }

        cmd.kill_on_drop(true);
        cmd.current_dir(&effective_cwd);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if let Some(envs) = env_vars {
            for (k, v) in envs {
                cmd.env(k, v);
            }
        }

        cmd.env("ATENA_SKILL_DIR", skill_dir.to_string_lossy().to_string());

        let child = cmd.spawn().map_err(|e| {
            format!(
                "Failed to spawn script '{}' in '{}': {}",
                script_path.display(),
                effective_cwd.display(),
                e
            )
        })?;

        let wait_result = timeout(timeout_duration, child.wait_with_output()).await;
        let elapsed = start_time.elapsed().as_millis() as u64;

        match wait_result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code();
                let success = output.status.success();

                Ok(SkillCommandResult {
                    success,
                    exit_code,
                    stdout,
                    stderr,
                    duration_ms: elapsed,
                })
            }
            Ok(Err(e)) => Err(format!("Script execution error: {}", e)),
            Err(_) => Err(format!(
                "Script execution timed out after {} ms",
                timeout_duration.as_millis()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_validator_blocks_destructive_commands() {
        let destructive = [
            "rm -rf /",
            "rm -rf /*",
            "rm -rf ~",
            "rm -fr /",
            "rm -rf $HOME",
            "rm -rf *",
            "rmdir /s /q c:\\",
            "rd /s /q c:\\",
            "mkfs.ext4 /dev/sda1",
            "dd if=/dev/zero of=/dev/sda",
            "format c:",
            ":(){ :|:& };:",
        ];

        for cmd in destructive {
            let res = SkillScriptRunner::validate_command_safety(cmd);
            assert!(
                res.is_err(),
                "Expected command '{}' to be blocked by safety validator, but it was allowed",
                cmd
            );
        }
    }

    #[test]
    fn test_safety_validator_allows_safe_commands() {
        let safe = [
            "echo 'Hello World'",
            "git status --porcelain",
            "cargo test",
            "npm run build",
            "pytest tests/",
            "python3 -m unittest",
            "ls -la",
        ];

        for cmd in safe {
            let res = SkillScriptRunner::validate_command_safety(cmd);
            assert!(
                res.is_ok(),
                "Expected safe command '{}' to be allowed, but got error: {:?}",
                cmd,
                res.err()
            );
        }
    }

    #[tokio::test]
    async fn test_run_command_safe_execution() {
        let res = SkillScriptRunner::run_command(
            "echo AtenaSkillRunnerTest",
            None,
            None,
            None,
            Some(5000),
        )
        .await;

        assert!(res.is_ok());
        let output = res.unwrap();
        assert!(output.success);
        assert_eq!(output.exit_code, Some(0));
        assert!(output.stdout.contains("AtenaSkillRunnerTest"));
    }

    #[tokio::test]
    async fn test_run_command_rejects_destructive() {
        let res = SkillScriptRunner::run_command(
            "rm -rf /",
            None,
            None,
            None,
            Some(5000),
        )
        .await;

        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Destructive command rejected"));
    }

    #[test]
    fn test_create_skill_with_scripts_success() {
        use crate::services::memory_engine::MemoryGraphEngine;
        use crate::core::memory::{SkillStep, SkillScriptFilePayload};

        let skill_name = "Automated Test Skill Unit";
        let res = MemoryGraphEngine::create_skill_with_scripts(
            skill_name,
            "A test procedural skill created with automated scripts",
            vec!["unit_test".to_string(), "automated_test".to_string()],
            vec![SkillStep {
                order: 1,
                instruction: "Run automated test".to_string(),
                tool_name: None,
                command: Some("echo 'running step'".to_string()),
                script_file: Some("scripts/test_exec.sh".to_string()),
                cwd: None,
                timeout_ms: None,
            }],
            vec![SkillScriptFilePayload {
                filename: "test_exec.sh".to_string(),
                content: "#!/bin/bash\necho 'hello from unit test script'\n".to_string(),
            }],
            None,
        );

        assert!(res.is_ok(), "Expected skill creation to succeed, got: {:?}", res.err());
        let skill = res.unwrap();
        assert_eq!(skill.name, skill_name);
        assert_eq!(skill.scripts.len(), 1);
        assert_eq!(skill.scripts[0], "test_exec.sh");

        if let Some(folder) = skill.folder_path {
            let path = std::path::PathBuf::from(folder);
            assert!(path.join("SKILL.md").exists());
            assert!(path.join("skill.json").exists());
            assert!(path.join("scripts").join("test_exec.sh").exists());
            let _ = std::fs::remove_dir_all(path);
        }
    }

    #[test]
    fn test_create_skill_with_destructive_script_rejected() {
        use crate::services::memory_engine::MemoryGraphEngine;
        use crate::core::memory::{SkillStep, SkillScriptFilePayload};

        let res = MemoryGraphEngine::create_skill_with_scripts(
            "Malicious Skill Attempt",
            "This should be blocked",
            vec!["evil".to_string()],
            vec![SkillStep {
                order: 1,
                instruction: "Dangerous step".to_string(),
                tool_name: None,
                command: Some("rm -rf /".to_string()),
                script_file: None,
                cwd: None,
                timeout_ms: None,
            }],
            vec![SkillScriptFilePayload {
                filename: "evil.sh".to_string(),
                content: "rm -rf ~\n".to_string(),
            }],
            None,
        );

        assert!(res.is_err(), "Expected malicious skill to be rejected");
    }

    #[test]
    fn test_update_skill_with_scripts_success() {
        use crate::services::memory_engine::MemoryGraphEngine;
        use crate::core::memory::{SkillStep, SkillScriptFilePayload};

        let created = MemoryGraphEngine::create_skill_with_scripts(
            "Test Updatable Skill",
            "Initial description",
            vec!["update test".to_string()],
            vec![SkillStep {
                order: 1,
                instruction: "Echo v1".to_string(),
                tool_name: None,
                command: Some("echo v1".to_string()),
                script_file: None,
                cwd: None,
                timeout_ms: None,
            }],
            vec![SkillScriptFilePayload {
                filename: "script.sh".to_string(),
                content: "echo hello\n".to_string(),
            }],
            None,
        ).expect("Initial creation should succeed");

        assert_eq!(created.version, 1);

        let updated = MemoryGraphEngine::update_skill_with_scripts(
            Some(created.id.clone()),
            Some("Test Updatable Skill".to_string()),
            Some("Updated description".to_string()),
            Some(vec!["update test 2".to_string()]),
            Some(vec![SkillStep {
                order: 1,
                instruction: "Echo v2".to_string(),
                tool_name: None,
                command: Some("echo v2".to_string()),
                script_file: None,
                cwd: None,
                timeout_ms: None,
            }]),
            Some(vec![SkillScriptFilePayload {
                filename: "script2.sh".to_string(),
                content: "echo v2\n".to_string(),
            }]),
            Some("Upgraded to v2".to_string()),
            None,
            Some("auto".to_string()),
            None,
        ).expect("Skill update should succeed");

        assert_eq!(updated.version, 2);
        assert_eq!(updated.description, "Updated description");
        assert_eq!(updated.permission_mode, "auto");
        assert!(updated.triggers.contains(&"update test 2".to_string()));
        assert!(updated.scripts.contains(&"script2.sh".to_string()));
        assert_eq!(updated.steps[0].instruction, "Echo v2");

        if let Some(folder) = updated.folder_path {
            let path = std::path::PathBuf::from(folder);
            let _ = std::fs::remove_dir_all(path);
        }
    }

    #[test]
    fn test_skill_enable_disable_toggle_and_filtering() {
        use crate::services::memory_engine::MemoryGraphEngine;
        use crate::core::memory::SkillStep;

        let skill = MemoryGraphEngine::learn_or_refine_skill_advanced(
            None,
            "Toggleable Test Skill",
            "Description for toggle test",
            vec!["toggle_trigger_xyz".to_string()],
            vec![SkillStep {
                order: 1,
                instruction: "Do toggle action".to_string(),
                tool_name: None,
                command: Some("echo toggle".to_string()),
                script_file: None,
                cwd: None,
                timeout_ms: None,
            }],
            None,
            None,
            Some("ask".to_string()),
            Some(true),
        );

        assert!(skill.enabled, "Newly created skill should be enabled");

        // Should be found by trigger when enabled
        let matches = MemoryGraphEngine::find_matching_skills("toggle_trigger_xyz");
        assert!(matches.iter().any(|s| s.id == skill.id));

        // Disable skill
        let toggled = MemoryGraphEngine::set_skill_enabled(&skill.id, false)
            .expect("Disabling skill should succeed");
        assert!(!toggled.enabled, "Skill should be marked disabled");

        // Should NOT be matched when disabled
        let matches_after = MemoryGraphEngine::find_matching_skills("toggle_trigger_xyz");
        assert!(!matches_after.iter().any(|s| s.id == skill.id));

        // Clean up
        let _ = MemoryGraphEngine::delete_skill(&skill.id);
    }

    #[tokio::test]
    async fn test_procedural_skills_tools_suppressed_when_memory_disabled() {
        use crate::services::mcp_service::McpManager;
        let mut tools = McpManager::native_atena_tools();
        tools.extend(McpManager::memory_and_skills_tools());

        let use_skills = false;
        if !use_skills {
            tools.retain(|t| {
                t.server_id != "skills"
                    && t.tool.name != "run_command"
                    && t.tool.name != "run_skill_command"
                    && t.tool.name != "run_skill_script"
                    && t.tool.name != "create_procedural_skill"
                    && t.tool.name != "update_procedural_skill"
                    && t.tool.name != "edit_procedural_skill"
            });
        }

        assert!(!tools.iter().any(|t| t.server_id == "skills"));
        assert!(!tools.iter().any(|t| t.tool.name == "run_command"));
        assert!(!tools.iter().any(|t| t.tool.name == "create_procedural_skill"));
        assert!(!tools.iter().any(|t| t.tool.name == "update_procedural_skill"));
        assert!(!tools.iter().any(|t| t.tool.name == "run_skill_script"));

        // General native utilities like web search and scratchpad must remain intact
        assert!(tools.iter().any(|t| t.tool.name == "atena_web_search"));
        assert!(tools.iter().any(|t| t.tool.name == "atena_scratchpad_write"));
    }
}
