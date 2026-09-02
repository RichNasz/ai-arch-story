import { Label } from '@patternfly/react-core';

interface Props {
  validation: 'unknown' | 'valid' | 'invalid';
  lastSaved: Date | null;
  outputUrl?: string;
  isPreviewStale: boolean;
}

export function StatusBar({ validation, lastSaved, outputUrl, isPreviewStale }: Props) {
  return (
    <footer style={{ height: 36, padding: '7px 16px', borderTop: '1px solid var(--pf-t--global--border--color--default)', display: 'flex', gap: 16, alignItems: 'center', fontSize: 13 }}>
      <Label isCompact color={validation === 'valid' ? 'green' : validation === 'invalid' ? 'red' : 'grey'}>
        {validation === 'valid' ? 'Valid' : validation === 'invalid' ? 'Invalid' : 'Not validated'}
      </Label>
      <span>{lastSaved ? `Saved ${lastSaved.toLocaleTimeString()}` : 'No changes saved this session'}</span>
      {isPreviewStale && <span>Preview needs re-layout</span>}
      {outputUrl && <a href={outputUrl} target="_blank" rel="noreferrer" style={{ marginLeft: 'auto' }}>Open rendered diagram</a>}
    </footer>
  );
}
