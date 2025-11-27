use std::path::Path;
use std::fs;

pub struct DomainValidator;

impl DomainValidator {
    pub fn validate_domain_name(domain: &str) -> Result<(), String> {
        if domain.trim().is_empty() {
            return Err("Domain name cannot be empty".to_string());
        }
        if domain.contains(' ') || !domain.contains('.') {
            return Err("Invalid domain format (must contain dots and no spaces)".to_string());
        }
        if domain.len() > 253 {
            return Err("Domain name too long".to_string());
        }
        // Basic check for valid chars (alphanumeric, dot, hyphen)
        if !domain.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-') {
             return Err("Domain contains invalid characters".to_string());
        }
        Ok(())
    }

    pub fn validate_proxy_target(target: &str) -> Result<(), String> {
        if target.trim().is_empty() {
            return Err("Target host cannot be empty".to_string());
        }
        
        if let Some(port_start) = target.rfind(':') {
            let port_str = &target[port_start + 1..];
            let port_clean = port_str.split('/').next().unwrap_or("");
            
            if let Ok(port) = port_clean.parse::<u16>() {
                if port == 0 { return Err("Port cannot be 0".to_string()); }
            } else {
                return Err("Invalid port number specified in target".to_string());
            }
        }

        Ok(())
    }

    pub fn validate_root_path(path: &str) -> Result<(), String> {
        if path.trim().is_empty() {
            return Err("Root path cannot be empty".to_string());
        }
        if !path.starts_with('/') {
            return Err("Root path must be absolute (start with /)".to_string());
        }
        if path.contains("..") {
            return Err("Directory traversal (..) is not allowed".to_string());
        }
        // Strict check: forbidden characters in path
        if path.chars().any(|c| "<>|?*".contains(c)) {
             return Err("Root path contains illegal characters".to_string());
        }
        Ok(())
    }

    pub fn validate_ssl_files(cert: &str, key: &str) -> Result<(), String> {
        if cert == "default" || key == "default" {
            return Ok(()); // Allow defaults provided by env
        }
        if !Path::new(cert).exists() {
            return Err(format!("SSL Certificate not found at path: {}", cert));
        }
        if !Path::new(key).exists() {
            return Err(format!("SSL Key not found at path: {}", key));
        }
        Ok(())
    }
}