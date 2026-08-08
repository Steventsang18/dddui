-- Rename the internal aionrs agent display name from "Rose CLI" to "DDDcli".
-- The agent_type (`aionrs`), id (`agent-aionrs`) and icon (`aion.svg`) remain
-- unchanged per project compliance red line. This migration renames rows that
-- were already migrated by 036 (which had set "Rose CLI"); fresh installs get
-- the correct name directly from the 001 seed.
UPDATE agent_metadata
SET name = 'DDDcli'
WHERE agent_type = 'aionrs'
  AND agent_source = 'internal'
  AND name = 'Rose CLI';
