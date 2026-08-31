import { useCallback, useEffect, useState } from 'react';
import { Button, Form, FormGroup, FormSelect, FormSelectOption, Spinner, TextInput } from '@patternfly/react-core';
import type { BrandingConfig } from '../types';
import { api } from '../api';

interface Props { onChange: () => void; onError: (message: string) => void }
const emptyBranding: BrandingConfig = { colors: {}, footer: { showGeneratedDate: false } };

function readAsDataUri(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result));
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}

export function BrandingTab({ onChange, onError }: Props) {
  const [branding, setBranding] = useState<BrandingConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const load = useCallback(async () => {
    try { setBranding(await api.getBranding()); } catch { setBranding(emptyBranding); }
  }, []);
  useEffect(() => { load(); }, [load]);

  const save = async () => {
    if (!branding) return;
    setSaving(true);
    try { setBranding(await api.putBranding(branding)); onChange(); }
    catch (e) { onError(`${e}`); }
    finally { setSaving(false); }
  };
  const asset = async (kind: 'logo' | 'favicon', file?: File) => {
    if (!branding || !file) return;
    try {
      const src = await readAsDataUri(file);
      setBranding(kind === 'logo' ? { ...branding, logo: { ...branding.logo, src } } : { ...branding, favicon: { src } });
    } catch (e) { onError(`Unable to read ${kind}: ${e}`); }
  };
  if (!branding) return <Spinner aria-label="Loading branding" />;

  return <Form style={{ padding: 8 }}>
    <FormGroup label="Organization name" fieldId="branding-organization"><TextInput id="branding-organization" value={branding.organization ?? ''} onChange={(_e, v) => setBranding({ ...branding, organization: v || undefined })} /></FormGroup>
    <FormGroup label="Logo" fieldId="branding-logo"><input id="branding-logo" type="file" accept="image/svg+xml,image/png,image/jpeg" onChange={e => asset('logo', e.target.files?.[0])} /></FormGroup>
    <FormGroup label="Logo alt text" fieldId="branding-logo-alt"><TextInput id="branding-logo-alt" value={branding.logo?.alt ?? ''} onChange={(_e, v) => setBranding({ ...branding, logo: { src: branding.logo?.src ?? '', ...branding.logo, alt: v || undefined } })} /></FormGroup>
    <FormGroup label="Logo placement" fieldId="branding-logo-placement"><FormSelect id="branding-logo-placement" value={branding.logo?.placement ?? 'header'} onChange={(_e, v) => setBranding({ ...branding, logo: { src: branding.logo?.src ?? '', ...branding.logo, placement: v as 'header' | 'corner' } })}><FormSelectOption value="header" label="Header" /><FormSelectOption value="corner" label="Corner" /></FormSelect></FormGroup>
    <FormGroup label="Logo height" fieldId="branding-logo-height"><input id="branding-logo-height" type="number" min={8} max={128} value={branding.logo?.height ?? 24} onChange={e => setBranding({ ...branding, logo: { src: branding.logo?.src ?? '', ...branding.logo, height: Number(e.target.value) } })} /></FormGroup>
    <FormGroup label="Primary color" fieldId="branding-primary"><TextInput id="branding-primary" value={branding.colors?.primary ?? ''} onChange={(_e, v) => setBranding({ ...branding, colors: { ...branding.colors, primary: v || undefined } })} /></FormGroup>
    <FormGroup label="Secondary color" fieldId="branding-secondary"><TextInput id="branding-secondary" value={branding.colors?.secondary ?? ''} onChange={(_e, v) => setBranding({ ...branding, colors: { ...branding.colors, secondary: v || undefined } })} /></FormGroup>
    <FormGroup label="Footer text" fieldId="branding-footer"><TextInput id="branding-footer" value={branding.footer?.text ?? ''} onChange={(_e, v) => setBranding({ ...branding, footer: { ...branding.footer, text: v || undefined } })} /></FormGroup>
    <FormGroup fieldId="branding-date"><label><input id="branding-date" type="checkbox" checked={branding.footer?.showGeneratedDate ?? false} onChange={e => setBranding({ ...branding, footer: { ...branding.footer, showGeneratedDate: e.target.checked } })} /> Show generated date</label></FormGroup>
    <FormGroup label="Favicon" fieldId="branding-favicon"><input id="branding-favicon" type="file" accept="image/png,image/x-icon,image/svg+xml" onChange={e => asset('favicon', e.target.files?.[0])} /></FormGroup>
    <Button variant="primary" onClick={save} isLoading={saving} isDisabled={saving}>Save branding</Button>
  </Form>;
}
