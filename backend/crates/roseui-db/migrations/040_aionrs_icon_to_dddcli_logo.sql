-- Point the internal aionrs agent (DDDcli) avatar at the new DDDUi brand logo
-- instead of the legacy aion.svg. The agent_type (`aionrs`), id (`agent-aionrs`)
-- and the aion.svg *file* itself remain unchanged per project compliance red
-- line; only the icon reference on the seeded row is updated so the conversation
-- bar shows the current brand mark.
UPDATE agent_metadata
SET icon = '/api/assets/logos/brand/app.png'
WHERE agent_type = 'aionrs'
  AND agent_source = 'internal'
  AND icon = '/api/assets/logos/brand/aion.svg';
