import { cleanup, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
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
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  beforeEach(() => {
    vi.mocked(api.listDiagrams).mockResolvedValue([]);
    vi.mocked(api.render).mockResolvedValue({ outputPath: 'diagrams/system-overview/output/system-overview.html' });
    vi.mocked(api.createDiagram).mockResolvedValue({
      version: '1.0', title: 'Checkout', theme: 'default', nodes: [], edges: [],
      flows: [], groups: [], metadata: {},
    });
    vi.mocked(api.render).mockResolvedValue({ outputPath: 'output/checkout.html' });
  });

  it('creates the first diagram from the empty state', async () => {
    const user = userEvent.setup();
    render(<App />);
    expect(await screen.findByRole('button', { name: 'Export HTML' })).toBeDisabled();
    await user.click(await screen.findByRole('button', { name: 'Create diagram' }));

    expect(screen.getByRole('dialog', { name: /^Create diagram/ })).toBeVisible();
    await user.type(screen.getByRole('textbox', { name: 'Title' }), 'Checkout');
    expect(screen.getByRole('textbox', { name: 'Name' })).toHaveValue('checkout');
    await user.click(screen.getByRole('button', { name: 'Create' }));

    await waitFor(() => expect(api.createDiagram).toHaveBeenCalledWith('checkout', 'Checkout'));
  });

  it('prevents duplicate re-layout requests while rendering', async () => {
    let resolveRender: ((value: { outputPath: string }) => void) | undefined;
    vi.mocked(api.listDiagrams).mockResolvedValue([
      { name: 'checkout', title: 'Checkout', hasOutput: true },
    ]);
    vi.mocked(api.render).mockImplementation(() => new Promise(resolve => { resolveRender = resolve; }));
    const user = userEvent.setup();
    render(<App />);

    const reLayoutButtons = await screen.findAllByRole('button', { name: 'Re-layout' });
    const reLayout = reLayoutButtons[reLayoutButtons.length - 1];
    await user.click(reLayout);

    expect(reLayout).toBeDisabled();
    resolveRender?.({ outputPath: 'output/checkout.html' });
  });

  it('places the editor split directly in the filled page section', async () => {
    vi.mocked(api.listDiagrams).mockResolvedValue([
      { name: 'checkout', title: 'Checkout', hasOutput: true },
    ]);
    const { container } = render(<App />);

    await screen.findByTitle('Diagram Preview');
    expect(container.querySelector('.pf-v6-c-page__main-body')).not.toBeInTheDocument();
  });

  it('synchronizes application state when rendering a newly created preview', async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.click(await screen.findByRole('button', { name: 'Create diagram' }));
    await user.type(screen.getByRole('textbox', { name: 'Title' }), 'Checkout');
    await user.click(screen.getByRole('button', { name: 'Create' }));

    const previewButtons = await screen.findAllByRole('button', { name: 'Render Preview' });
    await user.click(previewButtons[previewButtons.length - 1]);

    const renderedLinks = await screen.findAllByRole('link', { name: 'Open rendered diagram' });
    expect(renderedLinks[renderedLinks.length - 1]).toHaveAttribute('href', '/api/v1/diagrams/checkout/preview');
  });

  it('exports the rendered standalone HTML file for the selected diagram', async () => {
    vi.mocked(api.listDiagrams).mockResolvedValue([
      { name: 'system-overview', title: 'System Overview', hasOutput: false },
    ]);
    let downloaded: { href: string; filename: string } | undefined;
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(function (this: HTMLAnchorElement) {
      downloaded = { href: this.href, filename: this.download };
    });
    const user = userEvent.setup();

    render(<App />);
    await user.click(await screen.findByRole('button', { name: 'Export HTML' }));

    await waitFor(() => expect(downloaded).toEqual({
      href: 'http://localhost:3000/api/v1/diagrams/system-overview/preview',
      filename: 'system-overview.html',
    }));
    expect(api.render).toHaveBeenCalledWith('system-overview');
  });

  it('does not download an HTML file when rendering fails', async () => {
    vi.mocked(api.listDiagrams).mockResolvedValue([
      { name: 'system-overview', title: 'System Overview', hasOutput: false },
    ]);
    vi.mocked(api.render).mockRejectedValue(new Error('Validation failed'));
    const downloadClick = vi.spyOn(HTMLAnchorElement.prototype, 'click');
    const user = userEvent.setup();

    render(<App />);
    await user.click(await screen.findByRole('button', { name: 'Export HTML' }));

    expect(await screen.findByText('Render failed: Error: Validation failed')).toBeVisible();
    expect(downloadClick).not.toHaveBeenCalled();
  });

  it('prevents duplicate exports while HTML generation is in progress', async () => {
    vi.mocked(api.listDiagrams).mockResolvedValue([
      { name: 'system-overview', title: 'System Overview', hasOutput: false },
    ]);
    let finishRender: (result: { outputPath: string }) => void;
    vi.mocked(api.render).mockImplementation(() => new Promise(resolve => {
      finishRender = resolve;
    }));
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => {});
    const user = userEvent.setup();

    render(<App />);
    const exportButton = await screen.findByRole('button', { name: 'Export HTML' });
    await user.click(exportButton);

    await waitFor(() => expect(exportButton).toBeDisabled());
    finishRender!({ outputPath: 'diagrams/system-overview/output/system-overview.html' });
  });
});
