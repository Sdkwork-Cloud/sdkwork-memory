import { SdkworkAuthPage, type SdkworkAuthController } from "@sdkwork/auth-pc-react";

export function MemoryAuthRoutes({ controller }: { controller: SdkworkAuthController }) {
  return <SdkworkAuthPage basePath="/auth" controller={controller} homePath="/console" />;
}
