import { SdkworkAuthPage, type SdkworkAuthController, type SdkworkAuthHeaderSlotProps } from "@sdkwork/auth-pc-react";
import { BrainCircuit } from "lucide-react";

export function MemoryAuthRoutes({ controller }: { controller: SdkworkAuthController }) {
  return <SdkworkAuthPage appearance={{ slots: { Header: MemoryAuthHeader } }} basePath="/auth" controller={controller} homePath="/console" />;
}

function MemoryAuthHeader({ description, title }: SdkworkAuthHeaderSlotProps) {
  return (
    <header className="memory-auth-header">
      <div className="memory-auth-brand"><span><BrainCircuit size={18} /></span><strong>SDKWork Memory</strong></div>
      <h1>{title}</h1>
      <p>{description}</p>
    </header>
  );
}
