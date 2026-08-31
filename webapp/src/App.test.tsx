import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { App } from './App';
import { api } from './api';

vi.mock('./api', () => ({
  api: {
    listDiagrams: vi.fn(),
    createDiagram: vi.fn(),
    validate: vi.fn(),
    render: vi.fn(),
    getPreviewUrl: (name: string) => `/api/v1/diagrams/${name}/preview`,
  },
}));

describe('App diagram lifecycle', () => {
  beforeEach(() => {
    vi.mocked(api.listDiagrams).mockResolvedValue([]);
    vi.mocked(api.createDiagram).mockResolvedValue({
      version: '1.0', title: 'Checkout', theme: 'default', nodes: [], edges: [],
      flows: [], groups: [], metadata: {},
    });
  });

  it('creates the first diagram from the empty state', async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole('button', { name: 'Create diagram' }));

    expect(screen.getByRole('dialog', { name: /^Create diagram/ })).toBeVisible();
    await user.type(screen.getByRole('textbox', { name: 'Title' }), 'Checkout');
    expect(screen.getByRole('textbox', { name: 'Name' })).toHaveValue('checkout');
    await user.click(screen.getByRole('button', { name: 'Create' }));

    await waitFor(() => expect(api.createDiagram).toHaveBeenCalledWith('checkout', 'Checkout'));
  });
});
