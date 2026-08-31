import type { Diagram, DiagramListEntry, DiagramNode, DiagramEdge, DiagramFlow, DiagramGroup, ResolvedTypeRegistry, CustomTypes, BrandingConfig } from './types';

const BASE = '/api/v1';

async function request<T>(path: string, opts?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    ...opts,
    headers: { 'Content-Type': 'application/json', ...opts?.headers },
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body?.error?.message ?? `HTTP ${res.status}`);
  }
  if (res.status === 204) return undefined as T;
  return res.json();
}

export const api = {
  getBranding: () => request<BrandingConfig>('/shared/branding'),

  putBranding: (branding: BrandingConfig) =>
    request<BrandingConfig>('/shared/branding', { method: 'PUT', body: JSON.stringify(branding) }),

  listDiagrams: () =>
    request<{ diagrams: DiagramListEntry[] }>('/diagrams').then(r => r.diagrams),

  getDiagram: (name: string) =>
    request<Diagram>(`/diagrams/${name}`),

  createDiagram: (name: string, title: string) =>
    request<Diagram>('/diagrams', {
      method: 'POST',
      body: JSON.stringify({ name, title }),
    }),

  deleteDiagram: (name: string) =>
    request<void>(`/diagrams/${name}`, { method: 'DELETE' }),

  validate: (name: string) =>
    request<{ valid: boolean }>(`/diagrams/${name}/validate`, { method: 'POST' }),

  render: (name: string) =>
    request<{ outputPath: string }>(`/diagrams/${name}/render`, { method: 'POST' }),

  getPreviewUrl: (name: string) =>
    `${BASE}/diagrams/${name}/preview`,

  // Nodes
  listNodes: (name: string) =>
    request<DiagramNode[]>(`/diagrams/${name}/nodes`),

  addNode: (name: string, node: DiagramNode) =>
    request<DiagramNode>(`/diagrams/${name}/nodes`, {
      method: 'POST',
      body: JSON.stringify(node),
    }),

  updateNode: (name: string, id: string, node: DiagramNode) =>
    request<DiagramNode>(`/diagrams/${name}/nodes/${id}`, {
      method: 'PUT',
      body: JSON.stringify(node),
    }),

  deleteNode: (name: string, id: string) =>
    request<void>(`/diagrams/${name}/nodes/${id}`, { method: 'DELETE' }),

  // Edges
  listEdges: (name: string) =>
    request<DiagramEdge[]>(`/diagrams/${name}/edges`),

  addEdge: (name: string, edge: DiagramEdge) =>
    request<DiagramEdge>(`/diagrams/${name}/edges`, {
      method: 'POST',
      body: JSON.stringify(edge),
    }),

  updateEdge: (name: string, id: string, edge: DiagramEdge) =>
    request<DiagramEdge>(`/diagrams/${name}/edges/${id}`, {
      method: 'PUT',
      body: JSON.stringify(edge),
    }),

  deleteEdge: (name: string, id: string) =>
    request<void>(`/diagrams/${name}/edges/${id}`, { method: 'DELETE' }),

  // Flows
  listFlows: (name: string) =>
    request<DiagramFlow[]>(`/diagrams/${name}/flows`),

  addFlow: (name: string, flow: DiagramFlow) =>
    request<DiagramFlow>(`/diagrams/${name}/flows`, {
      method: 'POST',
      body: JSON.stringify(flow),
    }),

  updateFlow: (name: string, id: string, flow: DiagramFlow) =>
    request<DiagramFlow>(`/diagrams/${name}/flows/${id}`, {
      method: 'PUT',
      body: JSON.stringify(flow),
    }),

  deleteFlow: (name: string, id: string) =>
    request<void>(`/diagrams/${name}/flows/${id}`, { method: 'DELETE' }),

  // Groups
  listGroups: (name: string) =>
    request<DiagramGroup[]>(`/diagrams/${name}/groups`),

  addGroup: (name: string, group: DiagramGroup) =>
    request<DiagramGroup>(`/diagrams/${name}/groups`, {
      method: 'POST',
      body: JSON.stringify(group),
    }),

  updateGroup: (name: string, id: string, group: DiagramGroup) =>
    request<DiagramGroup>(`/diagrams/${name}/groups/${id}`, {
      method: 'PUT',
      body: JSON.stringify(group),
    }),

  deleteGroup: (name: string, id: string) =>
    request<void>(`/diagrams/${name}/groups/${id}`, { method: 'DELETE' }),

  // Types
  getResolvedTypes: (name?: string) =>
    request<ResolvedTypeRegistry>(name ? `/diagrams/${name}/types` : '/types'),

  getProjectTypes: () =>
    request<CustomTypes>('/project/types'),

  putProjectTypes: (types: CustomTypes) =>
    request<CustomTypes>('/project/types', {
      method: 'PUT',
      body: JSON.stringify(types),
    }),

  getDiagramCustomTypes: (name: string) =>
    request<CustomTypes>(`/diagrams/${name}/custom-types`),

  putDiagramCustomTypes: (name: string, types: CustomTypes) =>
    request<CustomTypes>(`/diagrams/${name}/custom-types`, {
      method: 'PUT',
      body: JSON.stringify(types),
    }),

  // Shapes
  listShapes: () =>
    request<{ shapes: { name: string }[] }>('/project/shapes').then(r => r.shapes),

  uploadShape: (name: string, svg: string) =>
    request<{ name: string }>('/project/shapes', {
      method: 'POST',
      body: JSON.stringify({ name, svg }),
    }),

  deleteShape: (name: string) =>
    request<void>(`/project/shapes/${name}`, { method: 'DELETE' }),
};
