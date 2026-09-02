import { FormGroup, TextInput, FormHelperText, HelperText, HelperTextItem } from '@patternfly/react-core';
import type { StyleOverride } from '../types';

interface Props {
  idPrefix: string;
  value?: StyleOverride;
  onChange: (value: StyleOverride | undefined) => void;
  includeShape?: boolean;
}

export function StyleFields({ idPrefix, value, onChange, includeShape = false }: Props) {
  const update = (field: keyof StyleOverride, raw: string) => {
    const next = { ...value };
    if (!raw) delete next[field];
    else if (field === 'opacity') next.opacity = Math.max(0, Math.min(1, Number(raw)));
    else Object.assign(next, { [field]: raw });
    onChange(Object.keys(next).length ? next : undefined);
  };

  return <>
    {(['color', 'background', 'border'] as const).map(field => (
      <FormGroup key={field} label={field[0].toUpperCase() + field.slice(1)} fieldId={`${idPrefix}-${field}`}>
        <TextInput id={`${idPrefix}-${field}`} value={value?.[field] ?? ''} onChange={(_e, v) => update(field, v)} aria-describedby={`${idPrefix}-${field}-help`} />
        <FormHelperText><HelperText><HelperTextItem id={`${idPrefix}-${field}-help`}>Enter a CSS color value, such as #0066cc or rgb(0, 102, 204).</HelperTextItem></HelperText></FormHelperText>
      </FormGroup>
    ))}
    <FormGroup label="Opacity" fieldId={`${idPrefix}-opacity`}>
      <TextInput id={`${idPrefix}-opacity`} type="number" min={0} max={1} step={0.1} value={value?.opacity ?? ''} onChange={(_e, v) => update('opacity', v)} />
    </FormGroup>
    {includeShape && <FormGroup label="Shape override" fieldId={`${idPrefix}-shape`}>
      <TextInput id={`${idPrefix}-shape`} value={value?.shape ?? ''} onChange={(_e, v) => update('shape', v)} placeholder="Optional shape name" />
    </FormGroup>}
  </>;
}
