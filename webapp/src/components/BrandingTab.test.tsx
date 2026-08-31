import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { api } from '../api';
import { BrandingTab } from './BrandingTab';

vi.mock('../api', () => ({ api: { getBranding: vi.fn(), putBranding: vi.fn() } }));

describe('BrandingTab', () => {
  beforeEach(() => {
    vi.mocked(api.getBranding).mockResolvedValue({ organization: 'Red Hat', colors: { primary: '#ee0000' }, footer: { showGeneratedDate: true } });
    vi.mocked(api.putBranding).mockImplementation(async value => value);
  });

  it('loads and saves project branding', async () => {
    const user = userEvent.setup();
    render(<BrandingTab onChange={() => {}} onError={() => {}} />);
    const organization = await screen.findByRole('textbox', { name: 'Organization name' });
    expect(organization).toHaveValue('Red Hat');
    await user.clear(organization); await user.type(organization, 'Acme');
    await user.click(screen.getByRole('button', { name: 'Save branding' }));
    await waitFor(() => expect(api.putBranding).toHaveBeenCalledWith(expect.objectContaining({ organization: 'Acme' })));
  });
});
