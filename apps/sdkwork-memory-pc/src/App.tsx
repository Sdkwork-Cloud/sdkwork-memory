import { SdkworkAuthGate, SdkworkAuthPage, useSdkworkAuthControllerState } from "@sdkwork/auth-pc-react";
import { MemoryAdminSdkProvider, createMemoryAdminResourceRegistry } from "@sdkwork/memory-pc-admin-core";
import { memoryModule as adminControlPlaneModule } from "@sdkwork/memory-pc-admin-control-plane";
import { memoryModule as adminEvaluationModule } from "@sdkwork/memory-pc-admin-evaluation";
import { memoryModule as adminGovernanceModule } from "@sdkwork/memory-pc-admin-governance";
import { memoryModule as adminKnowledgeGraphModule } from "@sdkwork/memory-pc-admin-knowledge-graph";
import { memoryModule as adminLearningModule } from "@sdkwork/memory-pc-admin-learning";
import { memoryModule as adminMemoryModule } from "@sdkwork/memory-pc-admin-memory";
import { memoryModule as adminOverviewModule } from "@sdkwork/memory-pc-admin-overview";
import { memoryModule as adminProvidersModule } from "@sdkwork/memory-pc-admin-providers";
import { memoryModule as adminRetrievalModule } from "@sdkwork/memory-pc-admin-retrieval";
import { MemoryAdminShell } from "@sdkwork/memory-pc-admin-shell";
import { MemoryI18nProvider, type MemoryPcModuleDefinition } from "@sdkwork/memory-pc-commons";
import { MemoryConsoleSdkProvider, createMemoryConsoleResourceRegistry } from "@sdkwork/memory-pc-console-core";
import { memoryModule as consoleGovernanceModule } from "@sdkwork/memory-pc-console-governance";
import { memoryModule as consoleKnowledgeModule } from "@sdkwork/memory-pc-console-knowledge";
import { memoryModule as consoleLearningModule } from "@sdkwork/memory-pc-console-learning";
import { memoryModule as consoleMemoryModule } from "@sdkwork/memory-pc-console-memory";
import { memoryModule as consoleOverviewModule } from "@sdkwork/memory-pc-console-overview";
import { memoryModule as consoleRetrievalModule } from "@sdkwork/memory-pc-console-retrieval";
import { MemoryConsoleShell } from "@sdkwork/memory-pc-console-shell";
import { assertUniqueMemoryModules, type MemoryLocale } from "@sdkwork/memory-pc-core";
import { useMemo, useState } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";

import type { BootstrappedMemoryPcRuntime } from "./bootstrap/runtime.ts";

const consoleModules = assertUniqueMemoryModules([
  consoleOverviewModule,
  consoleMemoryModule,
  consoleLearningModule,
  consoleRetrievalModule,
  consoleKnowledgeModule,
  consoleGovernanceModule,
] satisfies readonly MemoryPcModuleDefinition[]);

const adminModules = assertUniqueMemoryModules([
  adminOverviewModule,
  adminMemoryModule,
  adminLearningModule,
  adminRetrievalModule,
  adminProvidersModule,
  adminEvaluationModule,
  adminKnowledgeGraphModule,
  adminControlPlaneModule,
  adminGovernanceModule,
] satisfies readonly MemoryPcModuleDefinition[]);

const allModules = [...consoleModules, ...adminModules];

export function App({ runtime }: { runtime: BootstrappedMemoryPcRuntime }) {
  const [locale, setLocaleState] = useState<MemoryLocale>(runtime.config.defaultLocale);

  function setLocale(nextLocale: MemoryLocale): void {
    runtime.setLocale(nextLocale);
    setLocaleState(nextLocale);
  }

  return (
    <MemoryI18nProvider locale={locale} modules={allModules} setLocale={setLocale}>
      <BrowserRouter>
        <MemoryAuthenticatedApplication runtime={runtime} />
      </BrowserRouter>
    </MemoryI18nProvider>
  );
}

function MemoryAuthenticatedApplication({ runtime }: { runtime: BootstrappedMemoryPcRuntime }) {
  const authState = useSdkworkAuthControllerState(runtime.authController);
  const consoleRegistry = useMemo(() => createMemoryConsoleResourceRegistry(runtime.appClient), [runtime.appClient]);
  const adminRegistry = useMemo(() => createMemoryAdminResourceRegistry(runtime.adminClient), [runtime.adminClient]);
  const permissionScope = authState.session?.context?.permissionScope ?? [];
  const userLabel = authState.user?.displayName || authState.user?.email;
  const signOut = () => { void runtime.authController.signOut(); };

  return (
    <SdkworkAuthGate
      authBasePath="/auth"
      controller={runtime.authController}
      fallback={<div className="bootstrap-state" role="status">SDKWork Memory</div>}
      homePath="/console"
      protectedPrefixes={["/console", "/admin"]}
      renderAuthRoutes={<SdkworkAuthPage basePath="/auth" controller={runtime.authController} homePath="/console" />}
    >
      <MemoryConsoleSdkProvider client={runtime.appClient}>
        <MemoryAdminSdkProvider client={runtime.adminClient}>
          <Routes>
            <Route path="/console/*" element={<MemoryConsoleShell modules={consoleModules} permissionScope={permissionScope} registry={consoleRegistry} userLabel={userLabel} onSignOut={signOut} />} />
            <Route path="/admin/*" element={<MemoryAdminShell modules={adminModules} permissionScope={permissionScope} registry={adminRegistry} userLabel={userLabel} onSignOut={signOut} />} />
            <Route path="*" element={<Navigate to="/console" replace />} />
          </Routes>
        </MemoryAdminSdkProvider>
      </MemoryConsoleSdkProvider>
    </SdkworkAuthGate>
  );
}
