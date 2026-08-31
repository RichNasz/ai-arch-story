import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { api } from '../api';
import { TypesTab } from './TypesTab';

vi.mock('../api', () => ({ api: {
  getResolvedTypes: vi.fn(), getProjectTypes: vi.fn(), getDiagramCustomTypes: vi.fn(),
  putProjectTypes: vi.fn(), putDiagramCustomTypes: vi.fn(), listShapes: vi.fn(),
  uploadShape: vi.fn(),
} }));

describe('TypesTab scope handling', () => {
  beforeEach(() => {
    vi.mocked(api.getResolvedTypes).mockResolvedValue({ types: {
      'ml-model': { key: 'ml-model', label: 'ML Model', shape: 'hexagon', gv_shape: 'hexagon', accent_color: '#7C3AED', is_built_in: false },
    } });
    vi.mocked(api.getProjectTypes).mockResolvedValue({ types: {} });
    vi.mocked(api.getDiagramCustomTypes).mockResolvedValue({ types: { 'ml-model': { label: 'ML Model', shape: 'hexagon', accentColor: '#7C3AED' } } });
    vi.mocked(api.listShapes).mockResolvedValue([]);
    vi.mocked(api.putDiagramCustomTypes).mockImplementation(async (_name, types) => types);
  });

  it('saves an existing diagram type back to diagram scope', async () => {
    const user = userEvent.setup();
    render(<TypesTab diagramName="ml" onChange={() => {}} onError={() => {}} />);
    await user.click(await screen.findByRole('button', { name: 'Edit ML Model' }));
    const label = screen.getByRole('textbox', { name: 'Label' });
    await user.clear(label); await user.type(label, 'Model');
    await user.click(screen.getByRole('button', { name: 'Save' }));

    await waitFor(() => expect(api.putDiagramCustomTypes).toHaveBeenCalled());
    expect(api.putProjectTypes).not.toHaveBeenCalled();
  });
});
