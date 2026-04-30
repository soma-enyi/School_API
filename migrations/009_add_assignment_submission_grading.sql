-- Add grading fields to assignment submissions for mentor grading workflow
ALTER TABLE assignment_submissions
ADD COLUMN IF NOT EXISTS grade_score INTEGER CHECK (grade_score BETWEEN 0 AND 100),
ADD COLUMN IF NOT EXISTS feedback TEXT,
ADD COLUMN IF NOT EXISTS graded_by UUID REFERENCES users(id),
ADD COLUMN IF NOT EXISTS graded_at TIMESTAMPTZ;