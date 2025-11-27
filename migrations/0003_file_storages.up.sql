CREATE TABLE IF NOT EXISTS file_storages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    file_max_size BIGINT NOT NULL DEFAULT 5242880,
    compression_enabled BOOLEAN DEFAULT FALSE,
    img_resize_max_size INT DEFAULT 1920,
    img_default_format VARCHAR(10) DEFAULT 'webp',
    is_active BOOLEAN DEFAULT TRUE,
    api_key UUID DEFAULT uuid_generate_v4(),
    allowed_file_types VARCHAR(255) DEFAULT '*',
    quota_size BIGINT DEFAULT 1073741824,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);