export type BuiltInNodeType =
  | 'service' | 'datastore' | 'queue' | 'user' | 'external'
  | 'function' | 'gateway' | 'frontend' | 'storage' | 'generic';

export type NodeType = BuiltInNodeType | string;

export interface ResolvedType {
  key: string;
  label: string;
  shape: string;
  gv_shape: string;
  accent_color: string;
  description?: string;
  is_built_in: boolean;
}

export interface ResolvedTypeRegistry {
  types: Record<string, ResolvedType>;
}

export interface CustomTypeDef {
  label: string;
  shape?: string;
  gvShape?: string;
  accentColor?: string;
  description?: string;
}

export interface CustomTypes {
  types: Record<string, CustomTypeDef>;
}

export type EdgeDirection = 'one-way' | 'two-way' | 'none';
export type FlowSpeed = 'slow' | 'normal' | 'fast';
export type FlowAnimation = 'pulse' | 'particle' | 'highlight';

export interface StyleOverride {
  color?: string;
  background?: string;
  border?: string;
  opacity?: number;
  size?: string;
  shape?: string;
}

export interface DiagramNode {
  id: string;
  label: string;
  type?: NodeType;
  icon?: string;
  style?: StyleOverride;
  position?: { x: number; y: number };
  metadata: Record<string, unknown>;
}

export interface DiagramEdge {
  id: string;
  from: string;
  to: string;
  label?: string;
  direction?: EdgeDirection;
  style?: StyleOverride;
  metadata: Record<string, unknown>;
}

export interface FlowStep {
  edge: string;
  label?: string;
  description?: string;
  parallel?: boolean;
}

export interface FlowStyle {
  color?: string;
  speed?: FlowSpeed;
  animation?: FlowAnimation;
}

export interface DiagramFlow {
  id: string;
  label: string;
  description?: string;
  steps: FlowStep[];
  style?: FlowStyle;
  metadata: Record<string, unknown>;
}

export interface DiagramGroup {
  id: string;
  label: string;
  nodes: string[];
  groups: string[];
  style?: StyleOverride;
  metadata: Record<string, unknown>;
}

export interface Diagram {
  version: string;
  title: string;
  description?: string;
  theme: string;
  viewport?: { width: number; height: number };
  nodes: DiagramNode[];
  edges: DiagramEdge[];
  flows: DiagramFlow[];
  groups: DiagramGroup[];
  branding?: Record<string, unknown>;
  custom_types?: CustomTypes;
  metadata: Record<string, unknown>;
}

export interface DiagramListEntry {
  name: string;
  title: string;
  hasOutput: boolean;
}

export interface BrandingConfig {
  enabled?: boolean;
  organization?: string;
  logo?: { src: string; alt?: string; placement?: 'header' | 'corner'; height?: number };
  colors?: { primary?: string; secondary?: string };
  footer?: { text?: string; showGeneratedDate?: boolean };
  favicon?: { src: string };
  [key: string]: unknown;
}
