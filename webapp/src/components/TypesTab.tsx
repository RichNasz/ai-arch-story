import { useState, useEffect, useCallback } from 'react';
import {
  Button,
  DataList,
  DataListItem,
  DataListItemRow,
  DataListItemCells,
  DataListCell,
  DataListAction,
  Form,
  FormGroup,
  TextInput,
  FormSelect,
  FormSelectOption,
  Modal,
  ModalVariant,
  ModalBody,
  ModalFooter,
  ModalHeader,
  Label,
  FileUpload,
} from '@patternfly/react-core';
import { PencilAltIcon, TrashIcon, PlusCircleIcon } from '@patternfly/react-icons';
import type { ResolvedType, CustomTypeDef, CustomTypes } from '../types';
import { api } from '../api';
import { ConfirmDeleteModal } from './ConfirmDeleteModal';

const BUILT_IN_SHAPES = [
  'rounded-rect', 'cylinder', 'parallelogram', 'person', 'dashed-rect',
  'hexagon', 'diamond', 'browser', 'folder', 'rect',
];

interface Props {
  diagramName: string;
  onChange: () => void;
  onError: (msg: string) => void;
}

interface EditingType {
  key: string;
  label: string;
  shape: string;
  accentColor: string;
  description: string;
  scope: 'project' | 'diagram';
}

function emptyType(): EditingType {
  return { key: '', label: '', shape: 'rounded-rect', accentColor: '#94A3B8', description: '', scope: 'project' };
}

export function TypesTab({ diagramName, onChange, onError }: Props) {
  const [types, setTypes] = useState<ResolvedType[]>([]);
  const [editType, setEditType] = useState<EditingType | null>(null);
  const [isNew, setIsNew] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; label: string; itemType: string } | null>(null);
  const [origins, setOrigins] = useState<Record<string, 'built-in' | 'library' | 'project' | 'diagram'>>({});
  const [importedShapes, setImportedShapes] = useState<string[]>([]);
  const [shapeFileName, setShapeFileName] = useState('');
  const [shapeFileValue, setShapeFileValue] = useState('');

  const load = useCallback(async () => {
    try {
      const [registry, projectTypes, diagramTypes, shapes] = await Promise.all([
        api.getResolvedTypes(diagramName),
        api.getProjectTypes().catch((): CustomTypes => ({ types: {} })),
        api.getDiagramCustomTypes(diagramName),
        api.listShapes(),
      ]);
      setTypes(Object.values(registry.types).sort((a, b) => {
        if (a.is_built_in !== b.is_built_in) return a.is_built_in ? -1 : 1;
        return a.label.localeCompare(b.label);
      }));
      setImportedShapes(shapes.map(s => s.name));
      setOrigins(Object.fromEntries(Object.values(registry.types).map(t => [t.key,
        diagramTypes.types[t.key] ? 'diagram' : projectTypes.types[t.key] ? 'project' : t.is_built_in ? 'built-in' : 'library'
      ])));
    } catch (e) {
      onError(`${e}`);
    }
  }, [diagramName, onError]);

  useEffect(() => { load(); }, [load]);

  const openAdd = () => {
    setEditType(emptyType());
    setIsNew(true);
    setModalOpen(true);
  };

  const openEdit = (t: ResolvedType) => {
    setEditType({
      key: t.key, label: t.label, shape: t.shape, accentColor: t.accent_color,
      description: t.description ?? '', scope: origins[t.key] === 'diagram' ? 'diagram' : 'project',
    });
    setIsNew(false);
    setModalOpen(true);
  };

  const handleSave = async () => {
    if (!editType) return;
    try {
      const def: CustomTypeDef = {
        label: editType.label,
        shape: editType.shape,
        accentColor: editType.accentColor,
        description: editType.description || undefined,
      };

      if (editType.scope === 'diagram') {
        const existing = await api.getDiagramCustomTypes(diagramName);
        existing.types[editType.key] = def;
        await api.putDiagramCustomTypes(diagramName, existing);
      } else {
        let existing;
        try {
          existing = await api.getProjectTypes();
        } catch {
          existing = { types: {} };
        }
        existing.types[editType.key] = def;
        await api.putProjectTypes(existing);
      }

      setModalOpen(false);
      await load();
      onChange();
    } catch (e) {
      onError(`${e}`);
    }
  };

  const handleDelete = async (key: string) => {
    try {
      if (origins[key] === 'diagram') {
        const diagramTypes = await api.getDiagramCustomTypes(diagramName);
        delete diagramTypes.types[key];
        await api.putDiagramCustomTypes(diagramName, diagramTypes);
      } else {
        const projectTypes = await api.getProjectTypes();
        delete projectTypes.types[key];
        await api.putProjectTypes(projectTypes);
      }

      await load();
      onChange();
    } catch (e) {
      onError(`${e}`);
    }
  };

  const handleShapeUpload = async () => {
    const name = shapeFileName.replace(/\.svg$/i, '').toLowerCase().replace(/[^a-z0-9-]/g, '-');
    if (!name || !shapeFileValue.includes('<svg')) return;
    try {
      await api.uploadShape(name, shapeFileValue);
      setShapeFileName(''); setShapeFileValue('');
      await load();
    } catch (e) { onError(`${e}`); }
  };

  return (
    <>
      <Button variant="link" icon={<PlusCircleIcon />} onClick={openAdd} style={{ marginBottom: '8px' }}>
        Add Type
      </Button>
      <DataList aria-label="Types" isCompact>
        {types.map(t => (
          <DataListItem key={t.key} id={`type-${t.key}`}>
            <DataListItemRow>
              <DataListItemCells
                dataListCells={[
                  <DataListCell key="info">
                    <strong>{t.label}</strong>
                    <br />
                    <span style={{ fontSize: '12px', color: 'var(--pf-t--global--text--color--subtle)' }}>{t.key}</span>
                    {' '}
                    <Label
                      isCompact
                      style={{ backgroundColor: t.accent_color, color: '#fff' }}
                    >
                      {t.shape}
                    </Label>
                    {t.is_built_in && (
                      <Label isCompact color="blue" style={{ marginLeft: '4px' }}>built-in</Label>
                    )}
                  </DataListCell>,
                ]}
              />
              <DataListAction
                id={`action-type-${t.key}`}
                aria-label="Actions"
                aria-labelledby={`type-${t.key}`}
              >
                {origins[t.key] !== 'library' && <Button variant="plain" aria-label={`Edit ${t.label}`} onClick={() => openEdit(t)}>
                  <PencilAltIcon />
                </Button>}
                {(origins[t.key] === 'project' || origins[t.key] === 'diagram') && (
                  <Button variant="plain" isDanger aria-label={`${t.is_built_in ? 'Reset' : 'Delete'} ${t.label}`} onClick={() => setDeleteTarget({ id: t.key, label: t.label, itemType: t.is_built_in ? 'override' : 'type' })}>
                    <TrashIcon />
                  </Button>
                )}
              </DataListAction>
            </DataListItemRow>
          </DataListItem>
        ))}
      </DataList>

      <div style={{ marginTop: 16, paddingTop: 12, borderTop: '1px solid var(--pf-t--global--border--color--default)' }}>
        <strong>Import SVG shape</strong>
        <FileUpload
          id="shape-upload"
          type="text"
          value={shapeFileValue}
          filename={shapeFileName}
          filenamePlaceholder="Choose an SVG file"
          onFileInputChange={(_e, file) => setShapeFileName(file.name)}
          onDataChange={(_e, value) => setShapeFileValue(value)}
          onClearClick={() => { setShapeFileName(''); setShapeFileValue(''); }}
          browseButtonText="Choose SVG"
        />
        <Button variant="secondary" onClick={handleShapeUpload} isDisabled={!shapeFileName || !shapeFileValue.includes('<svg')} style={{ marginTop: 8 }}>Import shape</Button>
      </div>

      <ConfirmDeleteModal
        isOpen={!!deleteTarget}
        itemName={deleteTarget?.label ?? ''}
        itemType={deleteTarget?.itemType ?? 'type'}
        onConfirm={() => { if (deleteTarget) { handleDelete(deleteTarget.id); setDeleteTarget(null); } }}
        onCancel={() => setDeleteTarget(null)}
      />

      {modalOpen && editType && (
        <Modal
          variant={ModalVariant.small}
          isOpen
          onClose={() => setModalOpen(false)}
        >
          <ModalHeader title={isNew ? 'Add Custom Type' : `Edit: ${editType.label}`} />
          <ModalBody>
            <Form>
              <FormGroup label="Key (kebab-case, e.g. kafka-topic)" fieldId="type-key">
                <TextInput
                  id="type-key"
                  value={editType.key}
                  onChange={(_e, v) => setEditType({ ...editType, key: v.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '') })}
                  isDisabled={!isNew}
                  isRequired
                />
              </FormGroup>
              <FormGroup label="Label" fieldId="type-label" isRequired>
                <TextInput
                  id="type-label"
                  value={editType.label}
                  onChange={(_e, v) => setEditType({ ...editType, label: v })}
                  isRequired
                />
              </FormGroup>
              <FormGroup label="Shape" fieldId="type-shape">
                <FormSelect
                  id="type-shape"
                  value={editType.shape}
                  onChange={(_e, v) => setEditType({ ...editType, shape: v })}
                >
                  {[...BUILT_IN_SHAPES, ...importedShapes.filter(s => !BUILT_IN_SHAPES.includes(s))].map(s => (
                    <FormSelectOption key={s} value={s} label={s} />
                  ))}
                </FormSelect>
              </FormGroup>
              <div aria-label="Shape preview" style={{ height: 72, marginBottom: 16, display: 'grid', placeItems: 'center', border: `3px solid ${editType.accentColor}`, borderRadius: editType.shape === 'rounded-rect' ? 12 : 2, background: `${editType.accentColor}18` }}>
                {editType.label || editType.shape}
              </div>
              <FormGroup label="Accent Color" fieldId="type-color">
                <input
                  id="type-color"
                  type="color"
                  value={editType.accentColor}
                  onChange={e => setEditType({ ...editType, accentColor: e.target.value })}
                  style={{ width: '60px', height: '32px', border: 'none', cursor: 'pointer' }}
                />
                <TextInput
                  id="type-color-hex"
                  value={editType.accentColor}
                  onChange={(_e, v) => setEditType({ ...editType, accentColor: v })}
                  style={{ width: '100px', marginLeft: '8px', display: 'inline-block' }}
                />
              </FormGroup>
              <FormGroup label="Description" fieldId="type-desc">
                <TextInput
                  id="type-desc"
                  value={editType.description}
                  onChange={(_e, v) => setEditType({ ...editType, description: v })}
                  placeholder="Optional description for tooltips"
                />
              </FormGroup>
              {isNew && (
                <FormGroup label="Scope" fieldId="type-scope">
                  <FormSelect
                    id="type-scope"
                    value={editType.scope}
                    onChange={(_e, v) => setEditType({ ...editType, scope: v as 'project' | 'diagram' })}
                  >
                    <FormSelectOption value="project" label="Project-wide" />
                    <FormSelectOption value="diagram" label="Diagram only" />
                  </FormSelect>
                </FormGroup>
              )}
            </Form>
          </ModalBody>
          <ModalFooter>
            {!isNew && types.find(t => t.key === editType.key)?.is_built_in && origins[editType.key] !== 'built-in' && <Button variant="secondary" onClick={() => handleDelete(editType.key)}>Reset to default</Button>}
            <Button variant="primary" onClick={handleSave} isDisabled={!editType.key || !editType.label}>
              {isNew ? 'Add' : 'Save'}
            </Button>
            <Button variant="link" onClick={() => setModalOpen(false)}>Cancel</Button>
          </ModalFooter>
        </Modal>
      )}
    </>
  );
}
