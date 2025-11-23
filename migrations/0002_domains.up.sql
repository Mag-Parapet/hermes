CREATE TABLE IF NOT EXISTS domains (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    domain VARCHAR(255) NOT NULL UNIQUE,
    port INT NOT NULL UNIQUE, 
    max_body_size INT DEFAULT 50,
    is_ssl BOOLEAN DEFAULT TRUE,
    is_active BOOLEAN DEFAULT TRUE,
    ssl_certificate_path TEXT,
    ssl_certificate_key_path TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_domains_user_id ON domains(user_id);
CREATE INDEX IF NOT EXISTS idx_domains_domain ON domains(domain);
CREATE INDEX IF NOT EXISTS idx_domains_port ON domains(port);
CREATE INDEX IF NOT EXISTS idx_domains_is_active ON domains(is_active);