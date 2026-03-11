-- Expand application statuses to support the full pipeline:
--   pending → interview_invited → waitlisted → accepted
--   (rejected can happen at any stage)

-- Drop the old check constraint
ALTER TABLE applications DROP CONSTRAINT IF EXISTS applications_status_check;

-- Add updated check constraint with new statuses
ALTER TABLE applications ADD CONSTRAINT applications_status_check
    CHECK (status IN ('pending', 'interview_invited', 'waitlisted', 'accepted', 'rejected'));
