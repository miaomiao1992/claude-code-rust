//! Daemon mode

use crate::config::Config;
use crate::error::Result;
use crate::state::AppState;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tokio::net::UnixListener;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

/// Daemon mode
#[cfg(feature = "daemon")]
pub async fn run(config: Config, state: AppState) -> Result<()> {
    let daemon_manager = DaemonManager::new(config, state);
    daemon_manager.start_daemon().await
}

/// Daemon manager
#[derive(Debug)]
pub struct DaemonManager {
    /// Configuration
    config: Config,
    
    /// Application state
    state: AppState,
    
    /// Whether daemon is running
    is_running: bool,
    
    /// Socket path
    socket_path: String,
    
    /// Process ID file path
    pid_file: String,
}

impl DaemonManager {
    /// Create a new daemon manager
    pub fn new(config: Config, state: AppState) -> Self {
        let socket_path = config.daemon.socket_path.clone().unwrap_or("/tmp/claude-code.sock".to_string());
        let pid_file = config.daemon.pid_file.clone().unwrap_or("/tmp/claude-code.pid".to_string());
        
        Self {
            config,
            state,
            is_running: false,
            socket_path,
            pid_file,
        }
    }
    
    /// Start daemon
    pub async fn start(&mut self) -> Result<()> {
        self.start_daemon().await
    }
    
    /// Start daemon process
    pub async fn start_daemon(&self) -> Result<()> {
        // Check if daemon is already running
        if self.is_daemon_running()? {
            tracing::info!("Daemon is already running");
            return Ok(());
        }
        
        // Fork process
        self.fork_process()?;
        
        // Write PID file
        self.write_pid_file()?;
        
        // Start server
        self.start_server().await
    }
    
    /// Stop daemon
    pub async fn stop(&mut self) -> Result<()> {
        // Read PID from file
        if let Ok(pid) = self.read_pid_file() {
            // Send SIGTERM to process
            Command::new("kill").arg(pid.to_string()).output()?;
            
            // Remove PID file
            if Path::new(&self.pid_file).exists() {
                std::fs::remove_file(&self.pid_file)?;
            }
            
            // Remove socket file
            if Path::new(&self.socket_path).exists() {
                std::fs::remove_file(&self.socket_path)?;
            }
            
            self.is_running = false;
            tracing::info!("Daemon stopped");
        }
        
        Ok(())
    }
    
    /// Check if daemon is running
    fn is_daemon_running(&self) -> Result<bool> {
        if let Ok(pid) = self.read_pid_file() {
            // Check if process exists
            let output = Command::new("kill").arg("-0").arg(pid.to_string()).output()?;
            return Ok(output.status.success());
        }
        Ok(false)
    }
    
    /// Read PID from file
    fn read_pid_file(&self) -> Result<u32> {
        let content = std::fs::read_to_string(&self.pid_file)?;
        content.trim().parse().map_err(|e| e.into())
    }
    
    /// Write PID file
    fn write_pid_file(&self) -> Result<()> {
        let pid = std::process::id().to_string();
        let mut file = File::create(&self.pid_file)?;
        file.write_all(pid.as_bytes())?;
        Ok(())
    }
    
    /// Fork process
    fn fork_process(&self) -> Result<()> {
        // In production, we would use proper forking
        // For simplicity, we'll just continue execution
        Ok(())
    }
    
    /// Start server
    async fn start_server(&self) -> Result<()> {
        tracing::info!("Starting daemon server on {}", self.socket_path);
        
        // Remove existing socket if it exists
        if Path::new(&self.socket_path).exists() {
            std::fs::remove_file(&self.socket_path)?;
        }
        
        // Create Unix listener
        let listener = UnixListener::bind(&self.socket_path)?;
        
        // Accept connections
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    // Handle connection
                    tokio::spawn(async move {
                        // TODO: Handle connection
                    });
                }
                Err(e) => {
                    tracing::error!("Error accepting connection: {}", e);
                }
            }
        }
    }
    
    /// Get daemon status
    pub fn status(&self) -> Result<String> {
        if self.is_daemon_running()? {
            Ok("Daemon is running".to_string())
        } else {
            Ok("Daemon is not running".to_string())
        }
    }
    
    /// Restart daemon
    pub async fn restart(&mut self) -> Result<()> {
        self.stop().await?;
        self.start().await
    }
}

/// Daemon configuration extension
pub trait DaemonConfigExt {
    fn daemon_socket_path(&self) -> String;
    fn daemon_pid_file(&self) -> String;
}

impl DaemonConfigExt for Config {
    fn daemon_socket_path(&self) -> String {
        self.daemon.socket_path.clone().unwrap_or("/tmp/claude-code.sock".to_string())
    }
    
    fn daemon_pid_file(&self) -> String {
        self.daemon.pid_file.clone().unwrap_or("/tmp/claude-code.pid".to_string())
    }
}
