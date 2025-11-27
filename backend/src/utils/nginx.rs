use std::fs;
use std::path::Path;
use std::process::Command;
use crate::utils::nginx_templates::NginxTemplates;
#[cfg(unix)]
use std::os::unix::fs::symlink;

const SITES_AVAILABLE: &str = "/etc/nginx/sites-available";
const SITES_ENABLED: &str = "/etc/nginx/sites-enabled";

pub struct NginxManager;

impl NginxManager {
    
    pub fn deploy_site(
        domain_type: &str, 
        domain: &str, 
        nginx_target_host: Option<&str>, 
        nginx_root_path: Option<&str>,   
        bytes: i64,
        is_ssl: bool,
        cert_path: &str,
        key_path: &str,
        nginx_config_content: Option<&str>,
    ) -> Result<(), String> {
        
        let config_content = NginxTemplates::generate_config(
            domain_type, 
            domain, 
            nginx_target_host, 
            nginx_root_path, 
            bytes, 
            is_ssl, 
            cert_path, 
            key_path, 
            nginx_config_content
        ).map_err(|e| format!("Template generation failed: {}", e))?;

        // 2. Deployment - Platform Specific Logic

        // --- LINUX / UNIX (Production) ---
        #[cfg(unix)]
        {
            let available_path = format!("{}/{}", SITES_AVAILABLE, domain);
            let enabled_path = format!("{}/{}", SITES_ENABLED, domain);

            // A. Write to sites-available
            fs::write(&available_path, config_content)
                .map_err(|e| format!("Failed to write config file: {}", e))?;

            // B. Symlink to sites-enabled
            if Path::new(&enabled_path).exists() {
                let _ = fs::remove_file(&enabled_path);
            }
            
            symlink(&available_path, &enabled_path)
                .map_err(|e| format!("Failed to create symlink: {}", e))?;

            // C. Test Configuration (nginx -t)
            let test_output = Command::new("sudo")
                .arg("nginx")
                .arg("-t")
                .output()
                .map_err(|e| format!("Failed to execute nginx command: {}", e))?;

            if !test_output.status.success() {
                let _ = fs::remove_file(&enabled_path); // Rollback symlink
                // We keep the available file for debugging, or you could delete it too
                let error_msg = String::from_utf8_lossy(&test_output.stderr);
                return Err(format!("Nginx validation failed: {}", error_msg));
            }

            // D. Reload Nginx
            let reload_output = Command::new("systemctl")
                .arg("reload")
                .arg("nginx")
                .output()
                .map_err(|e| format!("Failed to reload nginx: {}", e))?;

            if !reload_output.status.success() {
                return Err("Failed to reload Nginx service".to_string());
            }
        }
        
        #[cfg(not(unix))]
        {
            println!("\n[WINDOWS DETECTED] Simulating Nginx Deployment for: {}", domain);
            println!("Configuration type: {}", domain_type);
            
            let _ = fs::create_dir_all("temp_nginx");
            let local_path = format!("temp_nginx/{}", domain);
            
            fs::write(&local_path, config_content)
                .map_err(|e| format!("Failed to write local simulation file: {}", e))?;
                
            println!("> Saved simulation file to: {}", local_path);
        }

        Ok(())
    }

    pub fn delete_site(domain: &str) -> Result<(), String> {
        
        #[cfg(unix)]
        {
            let available_path = format!("{}/{}", SITES_AVAILABLE, domain);
            let enabled_path = format!("{}/{}", SITES_ENABLED, domain);

            if Path::new(&enabled_path).exists() {
                let _ = fs::remove_file(&enabled_path);
            }
            if Path::new(&available_path).exists() {
                let _ = fs::remove_file(&available_path);
            }
            let _ = Command::new("systemctl").arg("reload").arg("nginx").output();
        }
        
        #[cfg(not(unix))]
        {
            println!("\n[WINDOWS DETECTED] Simulating Delete for: {}", domain);
            let local_path = format!("temp_nginx/{}", domain);
            if Path::new(&local_path).exists() {
                let _ = fs::remove_file(local_path);
                println!("> Deleted local simulation file");
            }
        }

        Ok(())
    }
}
