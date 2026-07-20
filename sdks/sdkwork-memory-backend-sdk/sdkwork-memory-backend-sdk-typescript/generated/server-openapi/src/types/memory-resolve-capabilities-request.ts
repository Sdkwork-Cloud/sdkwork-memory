export interface MemoryResolveCapabilitiesRequest {
  targetType: 'subject' | 'space' | 'binding' | 'memory';
  targetId: string;
}
