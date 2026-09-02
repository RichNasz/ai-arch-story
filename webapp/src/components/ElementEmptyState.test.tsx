import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { ElementEmptyState } from './ElementEmptyState';

describe('ElementEmptyState', () => {
  it('explains the next creation step and invokes the supplied action', async () => {
    const onCreate = vi.fn();
    const user = userEvent.setup();
    render(<ElementEmptyState title="No nodes yet" description="Add a node to start defining the architecture." actionLabel="Add node" onCreate={onCreate} />);

    expect(screen.getByText('Add a node to start defining the architecture.')).toBeVisible();
    await user.click(screen.getByRole('button', { name: 'Add node' }));
    expect(onCreate).toHaveBeenCalledOnce();
  });
});
