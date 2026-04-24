-- Per-student QR token tied to their attendance record.
-- Generated after sign-in; validated by mentor/admin to set attendance status = true.
CREATE TABLE IF NOT EXISTS student_attendance_qr (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    attendance_id UUID NOT NULL UNIQUE REFERENCES attendance(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    validated BOOLEAN NOT NULL DEFAULT FALSE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_student_attendance_qr_token ON student_attendance_qr(token);
