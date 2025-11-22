use std::fs;
use std::path::Path;
use std::process::Command;
use std::os::unix::fs::symlink;

const SITES_AVAILABLE: &str = "/etc/nginx/sites-available";
const SITES_ENABLED: &str = "/etc/nginx/sites-enabled";

pub struct NginxManager;

impl NginxManager {
    fn generate_template(domain: &str, port: i32, max_mb: i32) -> String {
        format!(
            r#"
server {{
    listen 80;
    server_name {domain} www.{domain};

    access_log /var/log/nginx/{domain}.access.log;
    error_log /var/log/nginx/{domain}.error.log;

    client_max_body_size {max_mb}M;

    location / {{
        proxy_pass http://127.0.0.1:{port};
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }}
}}
"#
        )
    }

    pub fn deploy_site(domain: &str, port: i32, max_mb: i32) -> Result<(), String> {
        let config_content = Self::generate_template(domain, port, max_mb);
        let available_path = format!("{}/{}", SITES_AVAILABLE, domain);
        let enabled_path = format!("{}/{}", SITES_ENABLED, domain);

        fs::write(&available_path, config_content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        if Path::new(&enabled_path).exists() {
            let _ = fs::remove_file(&enabled_path);
        }
        
        symlink(&available_path, &enabled_path)
            .map_err(|e| format!("Failed to create symlink: {}", e))?;

        let test_output = Command::new("nginx")
            .arg("-t")
            .output()
            .map_err(|e| format!("Failed to execute nginx command: {}", e))?;

        if !test_output.status.success() {
            let _ = fs::remove_file(&enabled_path);
            let error_msg = String::from_utf8_lossy(&test_output.stderr);
            return Err(format!("Nginx validation failed: {}", error_msg));
        }

        let reload_output = Command::new("systemctl")
            .arg("reload")
            .arg("nginx")
            .output()
            .map_err(|e| format!("Failed to reload nginx: {}", e))?;

        if !reload_output.status.success() {
            return Err("Failed to reload Nginx service".to_string());
        }

        Ok(())
    }

    pub fn delete_site(domain: &str) -> Result<(), String> {
        let available_path = format!("{}/{}", SITES_AVAILABLE, domain);
        let enabled_path = format!("{}/{}", SITES_ENABLED, domain);

        let _ = fs::remove_file(&enabled_path);
        let _ = fs::remove_file(&available_path);

        let _ = Command::new("systemctl").arg("reload").arg("nginx").output();

        Ok(())
    }
}