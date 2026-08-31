import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { api } from '../api';
import { NodesTab } from './NodesTab';
import { FlowsTab } from './FlowsTab';

vi.mock('../api', () => ({ api: {
  listNodes: vi.fn(), getResolvedTypes: vi.fn(), updateNode: vi.fn(),
  addNode: vi.fn(), deleteNode: vi.fn(),
  listFlows: vi.fn(), listEdges: vi.fn(), updateFlow: vi.fn(), addFlow: vi.fn(), deleteFlow: vi.fn(),
} }));

describe('element forms', () => {
  beforeEach(() => {
    vi.mocked(api.listNodes).mockResolvedValue([{ id: 'api', label: 'API', type: 'service', metadata: { owner: 'platform' }, style: { background: '#ffffff' } }]);
    vi.mocked(api.getResolvedTypes).mockResolvedValue({ types: { service: { key: 'service', label: 'Service', shape: 'rounded-rect', gv_shape: 'box', accent_color: '#3B82F6', is_built_in: true } } });
  });

  it('exposes node style and metadata fields', async () => {
    const user = userEvent.setup();
    render(<NodesTab diagramName="system" onChange={() => {}} onError={() => {}} />);
    await user.click(await screen.findByRole('button', { name: 'Edit API' }));

    expect(screen.getByRole('textbox', { name: 'Background' })).toHaveValue('#ffffff');
    expect(screen.getByDisplayValue('owner')).toBeVisible();
    expect(screen.getByDisplayValue('platform')).toBeVisible();
  });

  it('edits every flow-step field and provides ordering controls', async () => {
    vi.mocked(api.listFlows).mockResolvedValue([{ id: 'order', label: 'Order', metadata: {}, steps: [{ edge: 'submit', label: 'Submit', description: 'Send request', parallel: true }] }]);
    vi.mocked(api.listEdges).mockResolvedValue([{ id: 'submit', from: 'web', to: 'api', metadata: {} }]);
    const user = userEvent.setup();
    render(<FlowsTab diagramName="system" onChange={() => {}} onError={() => {}} />);
    await user.click(await screen.findByRole('button', { name: 'Edit Order' }));

    expect(screen.getByRole('textbox', { name: 'Step 1 description' })).toHaveValue('Send request');
    expect(screen.getByRole('checkbox', { name: 'Step 1 parallel' })).toBeChecked();
    expect(screen.getByRole('button', { name: 'Move step 1 down' })).toBeVisible();
  });
});
