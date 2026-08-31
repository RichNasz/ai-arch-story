import { Label } from '@patternfly/react-core';

interface Props {
  validation: 'unknown' | 'valid' | 'invalid';
  lastSaved: Date | null;
  outputUrl?: string;
}

export function StatusBar({ validation, lastSaved, outputUrl }: Props) {
  return (
    <footer style={{ height: 36, padding: '7px 16px', borderTop: '1px solid var(--pf-t--global--border--color--default)', display: 'flex', gap: 16, alignItems: 'center', fontSize: 13 }}>
      <Label isCompact color={validation === 'valid' ? 'green' : validation === 'invalid' ? 'red' : 'grey'}>
        {validation === 'valid' ? 'Valid' : validation === 'invalid' ? 'Invalid' : 'Not validated'}
      </Label>
      <span>{lastSaved ? `Saved ${lastSaved.toLocaleTimeString()}` : 'No changes saved this session'}</span>
      {outputUrl && <a href={outputUrl} target="_blank" rel="noreferrer" style={{ marginLeft: 'auto' }}>Open rendered diagram</a>}
    </footer>
  );
}
