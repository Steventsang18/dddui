import React, { Suspense } from 'react';
import { HashRouter, Navigate, Route, Routes, useLocation } from 'react-router-dom';
import AppLoader from '@renderer/components/layout/AppLoader';
import { useAuth } from '@renderer/hooks/context/AuthContext';
import { useProvidersQuery } from '@renderer/hooks/agent/useModelProviderList';
import { TEAM_MODE_ENABLED } from '@/common/config/constants';
import { isDesktopShell } from '@renderer/utils/platform';
const Conversation = React.lazy(() => import('@renderer/pages/conversation'));
const Guid = React.lazy(() => import('@renderer/pages/guid'));
const AgentSettings = React.lazy(() => import('@renderer/pages/settings/AgentSettings'));
const AgentRepairPage = React.lazy(() => import('@renderer/pages/settings/AgentSettings/AgentRepairPage'));
const AssistantSettings = React.lazy(() => import('@renderer/pages/settings/AssistantSettings'));
const SkillsSettings = React.lazy(() => import('@renderer/pages/settings/SkillsSettings/SkillsHubSettings'));
const SkillDetailPage = React.lazy(() => import('@renderer/pages/settings/SkillsSettings/SkillDetailPage'));
const ToolsSettings = React.lazy(() => import('@renderer/pages/settings/ToolsSettings'));
const IndustryTemplateSettings = React.lazy(() => import('@renderer/pages/settings/IndustryTemplateSettings'));
const AppearanceSettings = React.lazy(() => import('@renderer/pages/settings/AppearanceSettings'));
const SystemSettings = React.lazy(() => import('@renderer/pages/settings/SystemSettings'));
const WebuiSettings = React.lazy(() => import('@renderer/pages/settings/WebuiSettings'));
const PetSettings = React.lazy(() => import('@renderer/pages/settings/PetSettings'));
const ExtensionSettingsPage = React.lazy(() => import('@renderer/pages/settings/ExtensionSettingsPage'));
const CredentialsSettings = React.lazy(() => import('@renderer/pages/settings/CredentialsSettings'));
const LoginPage = React.lazy(() => import('@renderer/pages/login'));
const ComponentsShowcase = React.lazy(() => import('@renderer/pages/TestShowcase'));
const ScheduledTasksPage = React.lazy(() => import('@renderer/pages/cron/ScheduledTasksPage'));
const TaskDetailPage = React.lazy(() => import('@renderer/pages/cron/ScheduledTasksPage/TaskDetailPage'));
const TeamIndex = React.lazy(() => import('@renderer/pages/team'));
const WikiListPage = React.lazy(() => import('@renderer/pages/wiki/WikiListPage'));
const WikiDetailPage = React.lazy(() => import('@renderer/pages/wiki/WikiDetailPage'));
const TracePage = React.lazy(() => import('@renderer/components/conversation/TracePage'));
const Landing = React.lazy(() => import('@renderer/pages/Landing'));
const Setup = React.lazy(() => import('@renderer/pages/Setup'));

const withRouteFallback = (Component: React.LazyExoticComponent<React.ComponentType>) => (
  <Suspense fallback={<AppLoader />}>
    <Component />
  </Suspense>
);

// 桌面壳（Electron / Tauri）后端以 --local 模式运行，没有账号体系：
// 登录路由不注册，未认证守卫也不跳登录页，从入口根除登录界面。
// Desktop shells run the backend with --local (no accounts); the login
// screen is removed at the routing level entirely.
const DESKTOP_NO_LOGIN = isDesktopShell();

/**
 * Legacy `/settings/capabilities?tab=tools` deep links now map to the standalone
 * Tools page; everything else (skills tab or no tab) lands on the Skills page.
 */
const CapabilitiesRedirect: React.FC = () => {
  const { search } = useLocation();
  const tab = new URLSearchParams(search).get('tab');
  return <Navigate to={tab === 'tools' ? '/settings/tools' : '/settings/skills'} replace />;
};

const FirstRunGate: React.FC = () => {
  // Reuses the shared PROVIDERS_SWR_KEY + PROVIDERS_SWR_OPTIONS so the
  // providers cache is identical to every other consumer (no duplicate fetch).
  const { data: providers, isLoading } = useProvidersQuery();
  if (isLoading) return <AppLoader />;
  const hasProvider = Array.isArray(providers) && providers.length > 0;
  return <Navigate to={hasProvider ? '/guid' : '/setup'} replace />;
};

const ProtectedLayout: React.FC<{ layout: React.ReactElement }> = ({ layout }) => {
  const { status } = useAuth();

  if (status === 'checking') {
    return <AppLoader />;
  }

  if (status !== 'authenticated') {
    // 桌面壳免登录：即使 AuthContext 短路失效也不落入登录页。
    return DESKTOP_NO_LOGIN ? React.cloneElement(layout) : <Navigate to='/login' replace />;
  }

  return React.cloneElement(layout);
};

const PanelRoute: React.FC<{ layout: React.ReactElement }> = ({ layout }) => {
  const { status } = useAuth();

  return (
    <HashRouter>
      <Routes>
        <Route
          path='/login'
          element={
            status === 'authenticated' || DESKTOP_NO_LOGIN ? (
              <Navigate to='/guid' replace />
            ) : (
              withRouteFallback(LoginPage)
            )
          }
        />
        <Route path='/landing' element={withRouteFallback(Landing)} />
        <Route path='/setup' element={withRouteFallback(Setup)} />
        <Route index element={<FirstRunGate />} />
          <Route element={<ProtectedLayout layout={layout} />}>
          <Route path='/guid' element={withRouteFallback(Guid)} />
          <Route path='/conversation/:id' element={withRouteFallback(Conversation)} />
          <Route path='/conversation/:id/trace' element={withRouteFallback(TracePage)} />
          <Route
            path='/team/:id'
            element={TEAM_MODE_ENABLED ? withRouteFallback(TeamIndex) : <Navigate to='/guid' replace />}
          />
          <Route path='/settings/credentials' element={withRouteFallback(CredentialsSettings)} />
          {/* Model settings merged into Credentials center — redirect legacy deep links. */}
          <Route path='/settings/model' element={<Navigate to='/settings/credentials' replace />} />
          <Route path='/assistants' element={withRouteFallback(AssistantSettings)} />
          {/* Assistants moved out of Settings to a top-level entry; keep a redirect
              so old deep links / back-nav still land on the new page. */}
          <Route path='/settings/assistants' element={<Navigate to='/assistants' replace />} />
          <Route path='/settings/agent' element={withRouteFallback(AgentSettings)} />
          <Route path='/settings/agent/:id/repair' element={withRouteFallback(AgentRepairPage)} />
          {/* Skills and Tools are top-level settings entries. */}
          <Route path='/settings/skills' element={withRouteFallback(SkillsSettings)} />
          <Route path='/settings/skills/import-history' element={withRouteFallback(SkillsSettings)} />
          <Route path='/settings/skills/detail/:skillName' element={withRouteFallback(SkillDetailPage)} />
          <Route path='/settings/tools' element={withRouteFallback(ToolsSettings)} />
          <Route path='/settings/industry' element={withRouteFallback(IndustryTemplateSettings)} />
          {/* Legacy routes — the previous combined "Capabilities" page is now two pages. */}
          <Route path='/settings/capabilities' element={<CapabilitiesRedirect />} />
          <Route
            path='/settings/capabilities/skills/import-history'
            element={<Navigate to='/settings/skills/import-history' replace />}
          />
          <Route path='/settings/skills-hub' element={<Navigate to='/settings/skills' replace />} />
          <Route path='/settings/appearance' element={withRouteFallback(AppearanceSettings)} />
          <Route path='/settings/display' element={<Navigate to='/settings/appearance' replace />} />
          <Route path='/settings/webui' element={withRouteFallback(WebuiSettings)} />
          <Route path='/settings/pet' element={withRouteFallback(PetSettings)} />
          <Route path='/settings/system' element={withRouteFallback(SystemSettings)} />
          <Route path='/settings/about' element={withRouteFallback(SystemSettings)} />
          <Route path='/settings/ext/:tabId' element={withRouteFallback(ExtensionSettingsPage)} />
          <Route path='/settings' element={<Navigate to='/settings/agent' replace />} />
          <Route path='/test/components' element={withRouteFallback(ComponentsShowcase)} />
          <Route path='/wiki' element={withRouteFallback(WikiListPage)} />
          <Route path='/wiki/:id' element={withRouteFallback(WikiDetailPage)} />
          <Route path='/scheduled' element={withRouteFallback(ScheduledTasksPage)} />
          <Route path='/scheduled/:job_id' element={withRouteFallback(TaskDetailPage)} />
        </Route>
        <Route
          path='*'
          element={<Navigate to={status === 'authenticated' || DESKTOP_NO_LOGIN ? '/guid' : '/landing'} replace />}
        />
      </Routes>
    </HashRouter>
  );
};

export default PanelRoute;
