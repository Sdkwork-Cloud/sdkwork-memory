export interface MemoryResolveCapabilitiesRequest {
  tenantId: string;
  targetType: 'subject' | 'space' | 'binding' | 'memory';
  targetId: string;
}
