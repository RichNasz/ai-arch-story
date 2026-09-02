import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { PreviewPane } from './PreviewPane';

describe('PreviewPane', () => {
  it('shows an existing rendered diagram immediately', () => {
    render(
      <PreviewPane
        diagramName="system-overview"
        refreshKey={0}
        hasOutput
        isStale={false}
        onRender={async () => true}
      />,
    );

    expect(screen.getByTitle('Diagram Preview')).toHaveAttribute(
      'src',
      '/api/v1/diagrams/system-overview/preview?t=0',
    );
    expect(screen.queryByText('No preview available')).not.toBeInTheDocument();
  });
});
