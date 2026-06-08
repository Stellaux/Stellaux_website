CREATE TABLE guest_profiles (
    id        UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email     TEXT NOT NULL, --allow multiple guests per email, guest is identified by associated order
    session_id UUID NOT NULL REFERENCES session(id) ON DELETE CASCADE UNIQUE, --browser side session cookies
    device_fingerprint TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    converted_to_user_id UUID NULL REFERENCES users(id) ON DELETE SET NULL
    
);

