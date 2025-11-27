CREATE TABLE IF NOT EXISTS domains (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    domain VARCHAR(255) NOT NULL UNIQUE,
    
    client_max_body_size BIGINT DEFAULT 52428800, 
    
    is_ssl BOOLEAN DEFAULT TRUE,
    is_active BOOLEAN DEFAULT TRUE,

    domain_type VARCHAR(50) NOT NULL DEFAULT 'reverse_proxy',

    nginx_root_path VARCHAR(512),
    nginx_target_host VARCHAR(255),
    nginx_config_content TEXT,

    ssl_certificate_path TEXT,
    ssl_certificate_key_path TEXT,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);


CREATE INDEX IF NOT EXISTS idx_domains_user_id ON domains(user_id);
CREATE INDEX IF NOT EXISTS idx_domains_domain ON domains(domain);
CREATE INDEX IF NOT EXISTS idx_domains_type ON domains(domain_type);
CREATE INDEX IF NOT EXISTS idx_domains_is_active ON domains(is_active);