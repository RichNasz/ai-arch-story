import { beforeEach, describe, expect, it, vi } from 'vitest';
import { api } from './api';

describe('shape upload API contract', () => {
  beforeEach(() => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({
      ok: true,
      status: 201,
      json: async () => ({ name: 'cloud-native' }),
    }));
  });

  it('sends explicit name and SVG file multipart fields', async () => {
    // Sending JSON or manually setting multipart Content-Type must make this test fail.
    const svg = '<svg viewBox="0 0 10 10"><path d="M0 0h10v10H0z"/></svg>';

    await api.uploadShape('cloud-native', svg);

    expect(fetch).toHaveBeenCalledOnce();
    const [, options] = vi.mocked(fetch).mock.calls[0];
    expect(options?.headers).toBeUndefined();
    expect(options?.body).toBeInstanceOf(FormData);
    const form = options?.body as FormData;
    expect(form.get('name')).toBe('cloud-native');
    const file = form.get('file');
    expect(file).toBeInstanceOf(File);
    expect((file as File).name).toBe('cloud-native.svg');
    expect((file as File).type).toBe('image/svg+xml');
    expect(await (file as File).text()).toBe(svg);
  });
});
