-- Rename the internal aionrs agent display name from "Aion CLI" to "Rose CLI".
-- The agent_type (`aionrs`), id (`agent-aionrs`) and icon (`aion.svg`) remain
-- unchanged per project compliance red line.
UPDATE agent_metadata
SET name = 'Rose CLI'
WHERE agent_type = 'aionrs'
  AND agent_source = 'internal'
  AND name = 'Aion CLI';
