-- Applications table: stores course applications from unknown users (no account yet)
CREATE TABLE IF NOT EXISTS applications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    email VARCHAR(255) NOT NULL,
    course_id UUID NOT NULL REFERENCES courses(id) ON DELETE CASCADE,
    motivation TEXT NOT NULL,         -- "why did you choose this course?"
    experience TEXT,                  -- prior experience / background
    status VARCHAR(50) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'rejected')),
    reviewed_by UUID REFERENCES users(id),   -- admin who reviewed
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Prevent duplicate applications for the same email + course
CREATE UNIQUE INDEX IF NOT EXISTS idx_applications_email_course
    ON applications(email, course_id);

CREATE INDEX IF NOT EXISTS idx_applications_status ON applications(status);
CREATE INDEX IF NOT EXISTS idx_applications_course ON applications(course_id);
CREATE INDEX IF NOT EXISTS idx_applications_email ON applications(email);
