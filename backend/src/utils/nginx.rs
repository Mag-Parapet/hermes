use std::fs;
use std::path::Path;

pub struct NginxManager;

impl NginxManager {
    fn generate_template(
        domain: &str, 
        port: i32, 
        max_mb: i32, 
        is_ssl: bool,
        ssl_cert_path: Option<&str>,
        ssl_key_path: Option<&str>
    ) -> String {
        if is_ssl {
            // Use custom paths if provided, otherwise use Let's Encrypt defaults
            let default_cert = format!("/etc/letsencrypt/live/{}/fullchain.pem", domain);
            let default_key = format!("/etc/letsencrypt/live/{}/privkey.pem", domain);
            
            let cert_path = ssl_cert_path.unwrap_or(&default_cert);
            let key_path = ssl_key_path.unwrap_or(&default_key);

            format!(
                r#"# HTTPS server block
server {{
    server_name {domain};
    client_max_body_size {max_mb}M;

    access_log /var/log/nginx/{domain}.access.log;
    error_log /var/log/nginx/{domain}.error.log;

    location / {{
        proxy_pass http://127.0.0.1:{port};
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_cache_bypass $http_upgrade;
    }}

    # SSL configuration
    listen [::]:443 ssl;
    listen 443 ssl;
    ssl_certificate {cert_path};
    ssl_certificate_key {key_path};
    include /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;
}}

# HTTP to HTTPS redirect
server {{
    listen 80;
    listen [::]:80;
    server_name {domain};
    return 301 https://$server_name$request_uri;
}}
"#
            )
        } else {
            // Generate HTTP-only configuration
            format!(
                r#"server {{
    listen 80;
    listen [::]:80;
    server_name {domain};

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
        proxy_set_header X-Forwarded-Proto $scheme;
    }}
}}
"#
            )
        }
    }

    /// Deploy a site with optional SSL and custom certificate paths
    pub fn deploy_site(
        domain: &str, 
        port: i32, 
        max_mb: i32, 
        is_ssl: bool,
        ssl_cert_path: Option<&str>,
        ssl_key_path: Option<&str>
    ) -> Result<(), String> {
        // Validate SSL certificate paths when SSL is enabled
        if is_ssl {
            let default_cert = format!("/etc/letsencrypt/live/{}/fullchain.pem", domain);
            let default_key = format!("/etc/letsencrypt/live/{}/privkey.pem", domain);
            
            let cert_path = ssl_cert_path.unwrap_or(&default_cert);
            let key_path = ssl_key_path.unwrap_or(&default_key);
            
            #[cfg(unix)]
            {
                if !Path::new(cert_path).exists() {
                    return Err(format!(
                        "SSL certificate not found at: {}. Please provide valid certificate path.", 
                        cert_path
                    ));
                }
                if !Path::new(key_path).exists() {
                    return Err(format!(
                        "SSL private key not found at: {}. Please provide valid key path.", 
                        key_path
                    ));
                }
            }
        }

        let config_content = Self::generate_template(domain, port, max_mb, is_ssl, ssl_cert_path, ssl_key_path);

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let available_path = format!("{}/{}", SITES_AVAILABLE, domain);
            let enabled_path = format!("{}/{}", SITES_ENABLED, domain);

            // Write configuration file
            fs::write(&available_path, config_content)
                .map_err(|e| format!("Failed to write config file: {}", e))?;

            // Remove existing symlink if present
            if Path::new(&enabled_path).exists() {
                let _ = fs::remove_file(&enabled_path);
            }
            
            // Create symlink
            symlink(&available_path, &enabled_path)
                .map_err(|e| format!("Failed to create symlink: {}", e))?;

            // Test nginx configuration
            let test_output = Command::new("nginx")
                .arg("-t")
                .output()
                .map_err(|e| format!("Failed to execute nginx command: {}", e))?;

            if !test_output.status.success() {
                // Rollback on failure
                let _ = fs::remove_file(&enabled_path);
                let error_msg = String::from_utf8_lossy(&test_output.stderr);
                return Err(format!("Nginx validation failed: {}", error_msg));
            }

            // Reload nginx
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

        #[cfg(not(unix))]
        {
            println!("\n[WINDOWS DETECTED] Simulating Nginx Deployment for: {}", domain);
            println!("SSL enabled: {}", is_ssl);
            if is_ssl {
                println!("Certificate: {}", ssl_cert_path.unwrap_or("default"));
                println!("Private Key: {}", ssl_key_path.unwrap_or("default"));
            }
            println!("1. Would write config to /etc/nginx/sites-available/{}", domain);
            println!("2. Would symlink to /etc/nginx/sites-enabled/{}", domain);
            println!("3. Would run 'nginx -t' and 'systemctl reload nginx'");

            let _ = fs::create_dir_all("temp_nginx");
            let local_path = format!("temp_nginx/{}", domain);
            fs::write(&local_path, config_content)
                .map_err(|e| format!("Failed to write local simulation file: {}", e))?;
            
            println!("> Saved simulation file to: {}", local_path);
            
            Ok(())
        }
    }

    /// Delete a site
    pub fn delete_site(domain: &str) -> Result<(), String> {
        
        #[cfg(unix)]
        {
            let available_path = format!("{}/{}", SITES_AVAILABLE, domain);
            let enabled_path = format!("{}/{}", SITES_ENABLED, domain);

            // Remove symlink and config file
            let _ = fs::remove_file(&enabled_path);
            let _ = fs::remove_file(&available_path);
            
            // Reload nginx
            let _ = Command::new("systemctl").arg("reload").arg("nginx").output();
            
            Ok(())
        }

        #[cfg(not(unix))]
        {
            println!("\n[WINDOWS DETECTED] Simulating Delete for: {}", domain);
            let local_path = format!("temp_nginx/{}", domain);
            if Path::new(&local_path).exists() {
                let _ = fs::remove_file(local_path);
                println!("> Deleted local simulation file");
            }
            Ok(())
        }
    }

    /// Setup SSL certificates using certbot (optional utility method)
    #[cfg(unix)]
    pub fn setup_ssl(domain: &str, email: &str) -> Result<(), String> {
        let output = Command::new("certbot")
            .arg("certonly")
            .arg("--nginx")
            .arg("-d")
            .arg(domain)
            .arg("--email")
            .arg(email)
            .arg("--agree-tos")
            .arg("--non-interactive")
            .output()
            .map_err(|e| format!("Failed to run certbot: {}", e))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Certbot failed: {}", error_msg));
        }

        Ok(())
    }
}