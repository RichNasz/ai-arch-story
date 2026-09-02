import { useState } from 'react';
import {
  EmptyState,
  EmptyStateBody,
  EmptyStateActions,
  EmptyStateFooter,
  Button,
  Alert,
} from '@patternfly/react-core';
import { EyeIcon } from '@patternfly/react-icons';
import { api } from '../api';

interface Props {
  diagramName: string;
  refreshKey: number;
  hasOutput: boolean;
  isStale: boolean;
  onRender: () => Promise<void>;
}

export function PreviewPane({ diagramName, refreshKey, hasOutput, isStale, onRender }: Props) {
  const [hasRendered, setHasRendered] = useState(false);
  const [rendering, setRendering] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const previewUrl = `${api.getPreviewUrl(diagramName)}?t=${refreshKey}`;

  const handleRenderFirst = async () => {
    setRendering(true);
    setError(null);
    try {
      await onRender();
      setHasRendered(true);
    } catch (e) {
      setError(`${e}`);
    } finally {
      setRendering(false);
    }
  };

  if (!hasOutput && !hasRendered && refreshKey === 0) {
    return (
      <div style={{ height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <EmptyState headingLevel="h3" titleText="No preview available" icon={EyeIcon}>
          <EmptyStateBody>
            {error ?? 'Click Render in the toolbar to generate a preview, or click below.'}
          </EmptyStateBody>
          <EmptyStateFooter>
            <EmptyStateActions>
              <Button
                variant="primary"
                onClick={handleRenderFirst}
                isLoading={rendering}
                isDisabled={rendering}
              >
                Render Preview
              </Button>
            </EmptyStateActions>
          </EmptyStateFooter>
        </EmptyState>
      </div>
    );
  }

  return (
    <div style={{ height: '100%', minHeight: 0, position: 'relative' }}>
      {isStale && <Alert isInline variant="info" title="Preview is out of date. Select Re-layout to update it." style={{ position: 'absolute', inset: '16px 16px auto', zIndex: 1 }} />}
      <iframe
      src={previewUrl}
      title="Diagram Preview"
      style={{
        width: '100%',
        height: '100%',
        border: 'none',
        backgroundColor: '#fff',
        display: 'block',
      }}
      />
    </div>
  );
}
