-- Add UNIQUE constraint on group names to prevent duplicates.
CREATE UNIQUE INDEX IF NOT EXISTS idx_groups_name_unique ON groups (name);

-- Add UNIQUE constraint on organization names to prevent duplicates.
CREATE UNIQUE INDEX IF NOT EXISTS idx_organizations_name_unique ON organizations (name);
