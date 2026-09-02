import { useState } from 'react';
import {
  EmptyState,
  EmptyStateBody,
  EmptyStateActions,
  EmptyStateFooter,
  Button,
} from '@patternfly/react-core';
import { EyeIcon } from '@patternfly/react-icons';
import { api } from '../api';

interface Props {
  diagramName: string;
  refreshKey: number;
  hasOutput: boolean;
}

export function PreviewPane({ diagramName, refreshKey, hasOutput }: Props) {
  const [hasRendered, setHasRendered] = useState(false);
  const [rendering, setRendering] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const previewUrl = `${api.getPreviewUrl(diagramName)}?t=${refreshKey}`;

  const handleRenderFirst = async () => {
    setRendering(true);
    setError(null);
    try {
      await api.render(diagramName);
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
            {error ?? 'Click Export HTML in the toolbar to generate a preview and download it, or click below.'}
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
    <iframe
      src={previewUrl}
      title="Diagram Preview"
      style={{
        width: '100%',
        height: '100%',
        border: 'none',
        backgroundColor: '#fff',
      }}
    />
  );
}
