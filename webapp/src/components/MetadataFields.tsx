import { Button, FormGroup, TextInput } from '@patternfly/react-core';
import { PlusCircleIcon, TrashIcon } from '@patternfly/react-icons';

interface Props {
  idPrefix: string;
  value: Record<string, unknown>;
  onChange: (value: Record<string, unknown>) => void;
}

export function MetadataFields({ idPrefix, value, onChange }: Props) {
  const entries = Object.entries(value);
  const replace = (index: number, key: string, nextValue: unknown) => {
    const next: Record<string, unknown> = {};
    entries.forEach(([oldKey, oldValue], i) => { next[i === index ? key : oldKey] = i === index ? nextValue : oldValue; });
    onChange(next);
  };
  return <FormGroup label="Metadata" fieldId={`${idPrefix}-metadata`}>
    {entries.map(([key, item], index) => <div key={`${key}-${index}`} style={{ display: 'flex', gap: 8, marginBottom: 8 }}>
      <TextInput aria-label={`Metadata key ${index + 1}`} value={key} onChange={(_e, v) => replace(index, v, item)} placeholder="Key" />
      <TextInput aria-label={`Metadata value ${index + 1}`} value={typeof item === 'string' ? item : JSON.stringify(item)} onChange={(_e, v) => replace(index, key, v)} placeholder="Value" />
      <Button variant="plain" isDanger aria-label={`Remove metadata ${key}`} onClick={() => onChange(Object.fromEntries(entries.filter((_, i) => i !== index)))}><TrashIcon /></Button>
    </div>)}
    <Button variant="link" size="sm" icon={<PlusCircleIcon />} onClick={() => onChange({ ...value, [`key-${entries.length + 1}`]: '' })}>Add metadata</Button>
  </FormGroup>;
}
