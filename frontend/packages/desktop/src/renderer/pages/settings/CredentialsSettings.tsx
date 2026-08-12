/**
 * @license
 * Copyright 2025 DoDidDoneUi (dodiddoneui.com)
 * SPDX-License-Identifier: Apache-2.0
 */

import React from 'react';
import ModelModalContent from '@/renderer/components/settings/SettingsModal/contents/ModelModalContent';
import SettingsPageWrapper from './components/SettingsPageWrapper';

/**
 * Credentials center — a first-class "Connect & Credentials" entry that aggregates
 * cloud model API keys and local model providers (Ollama / LM Studio). It reuses the
 * existing ModelModalContent (provider/key management) with an overridden page header,
 * so no backend or data-layer changes are introduced.
 */
const CredentialsSettings: React.FC = () => {
  return (
    <SettingsPageWrapper contentClassName='max-w-1100px'>
      <ModelModalContent
        titleKey='settings.credentials'
        descriptionKey='settings.credentialsDescription'
      />
    </SettingsPageWrapper>
  );
};

export default CredentialsSettings;
