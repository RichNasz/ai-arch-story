import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { EditorSidebar } from './EditorSidebar';

vi.mock('./NodesTab', () => ({ NodesTab: () => null }));
vi.mock('./EdgesTab', () => ({ EdgesTab: () => null }));
vi.mock('./FlowsTab', () => ({ FlowsTab: () => null }));
vi.mock('./GroupsTab', () => ({ GroupsTab: () => null }));
vi.mock('./TypesTab', () => ({ TypesTab: () => null }));
vi.mock('./BrandingTab', () => ({ BrandingTab: () => null }));

describe('EditorSidebar', () => {
  it('uses vertical navigation so every editor section remains discoverable', () => {
    render(<EditorSidebar diagramName="system" onChange={() => {}} onError={() => {}} />);

    expect(screen.getByRole('tablist').closest('.pf-v6-c-tabs')).toHaveClass('pf-m-vertical');
    expect(screen.getByRole('tab', { name: 'Branding' })).toBeVisible();
  });
});
