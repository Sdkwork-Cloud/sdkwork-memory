import { useSdkworkAuthControllerState, type SdkworkAuthController } from "@sdkwork/auth-pc-react";
import { useEffect, useState, type ReactNode } from "react";
import { Navigate, useLocation } from "react-router-dom";

const AUTH_BOOTSTRAP_TIMEOUT_MS = 6_000;

export interface MemoryAuthGateProps {
  authRoutes: ReactNode;
  children: ReactNode;
  controller: SdkworkAuthController;
}

export function MemoryAuthGate({ authRoutes, children, controller }: MemoryAuthGateProps) {
  const location = useLocation();
  const state = useSdkworkAuthControllerState(controller);
  const onAuthRoute = location.pathname === "/auth" || location.pathname.startsWith("/auth/");
  const [bootstrapComplete, setBootstrapComplete] = useState(state.isBootstrapped);

  useEffect(() => {
    if (onAuthRoute || state.isBootstrapped) {
      setBootstrapComplete(true);
      return;
    }
    let active = true;
    const timeout = globalThis.setTimeout(() => {
      if (active) setBootstrapComplete(true);
    }, AUTH_BOOTSTRAP_TIMEOUT_MS);
    void controller.bootstrap().finally(() => {
      globalThis.clearTimeout(timeout);
      if (active) setBootstrapComplete(true);
    });
    return () => {
      active = false;
      globalThis.clearTimeout(timeout);
    };
  }, [controller, onAuthRoute, state.isBootstrapped]);

  if (onAuthRoute) return <>{authRoutes}</>;
  if (!bootstrapComplete) return <div className="bootstrap-state" role="status">SDKWork Memory</div>;
  if (!state.isAuthenticated) {
    const redirect = encodeURIComponent(`${location.pathname}${location.search}`);
    return <Navigate to={`/auth/login?redirect=${redirect}`} replace />;
  }
  return <>{children}</>;
}
