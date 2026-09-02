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
  TextArea,
  FormSelect,
  FormSelectOption,
  Modal,
  ModalVariant,
  ModalBody,
  ModalFooter,
  ModalHeader,
  Label,
} from '@patternfly/react-core';
import { PencilAltIcon, TrashIcon, PlusCircleIcon, ArrowUpIcon, ArrowDownIcon } from '@patternfly/react-icons';
import type { DiagramFlow, DiagramEdge, FlowStep, FlowSpeed, FlowAnimation } from '../types';
import { api } from '../api';
import { ConfirmDeleteModal } from './ConfirmDeleteModal';
import { MetadataFields } from './MetadataFields';
import { ElementEmptyState } from './ElementEmptyState';

const SPEEDS: FlowSpeed[] = ['slow', 'normal', 'fast'];
const ANIMATIONS: FlowAnimation[] = ['pulse', 'particle', 'highlight'];

interface Props {
  diagramName: string;
  onChange: () => void;
  onError: (msg: string) => void;
}

function emptyFlow(): DiagramFlow {
  return { id: '', label: '', steps: [], metadata: {} };
}

export function FlowsTab({ diagramName, onChange, onError }: Props) {
  const [flows, setFlows] = useState<DiagramFlow[]>([]);
  const [edges, setEdges] = useState<DiagramEdge[]>([]);
  const [editFlow, setEditFlow] = useState<DiagramFlow | null>(null);
  const [isNew, setIsNew] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; label: string } | null>(null);

  const load = useCallback(async () => {
    try {
      const [f, e] = await Promise.all([
        api.listFlows(diagramName),
        api.listEdges(diagramName),
      ]);
      setFlows(f);
      setEdges(e);
    } catch (e) {
      onError(`${e}`);
    }
  }, [diagramName, onError]);

  useEffect(() => { load(); }, [load]);

  const openAdd = () => {
    setEditFlow(emptyFlow());
    setIsNew(true);
    setModalOpen(true);
  };

  const openEdit = (flow: DiagramFlow) => {
    setEditFlow(JSON.parse(JSON.stringify(flow)));
    setIsNew(false);
    setModalOpen(true);
  };

  const handleSave = async () => {
    if (!editFlow) return;
    try {
      if (isNew) {
        const id = editFlow.id || editFlow.label.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
        await api.addFlow(diagramName, { ...editFlow, id });
      } else {
        await api.updateFlow(diagramName, editFlow.id, editFlow);
      }
      setModalOpen(false);
      await load();
      onChange();
    } catch (e) {
      onError(`${e}`);
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteFlow(diagramName, id);
      await load();
      onChange();
    } catch (e) {
      onError(`${e}`);
    }
  };

  const addStep = () => {
    if (!editFlow) return;
    setEditFlow({
      ...editFlow,
      steps: [...editFlow.steps, { edge: '', label: '' }],
    });
  };

  const updateStep = (idx: number, field: keyof FlowStep, value: string | boolean) => {
    if (!editFlow) return;
    const steps = [...editFlow.steps];
    steps[idx] = { ...steps[idx], [field]: value || undefined };
    setEditFlow({ ...editFlow, steps });
  };

  const removeStep = (idx: number) => {
    if (!editFlow) return;
    const steps = editFlow.steps.filter((_, i) => i !== idx);
    setEditFlow({ ...editFlow, steps });
  };

  const moveStep = (idx: number, offset: number) => {
    if (!editFlow) return;
    const target = idx + offset;
    if (target < 0 || target >= editFlow.steps.length) return;
    const steps = [...editFlow.steps];
    [steps[idx], steps[target]] = [steps[target], steps[idx]];
    setEditFlow({ ...editFlow, steps });
  };

  const edgeLabel = (id: string) => {
    const e = edges.find(e => e.id === id);
    return e ? `${e.from} → ${e.to}${e.label ? ` (${e.label})` : ''}` : id;
  };

  return (
    <>
      {flows.length > 0 && <Button variant="link" icon={<PlusCircleIcon />} onClick={openAdd} style={{ marginBottom: '8px' }}>
        Add Flow
      </Button>}
      {flows.length === 0 ? <ElementEmptyState title="No flows yet" description="Add a flow to explain how work moves through the architecture." actionLabel="Add flow" onCreate={openAdd} /> : <DataList aria-label="Flows" isCompact>
        {flows.map(flow => (
          <DataListItem key={flow.id} id={flow.id}>
            <DataListItemRow>
              <DataListItemCells
                dataListCells={[
                  <DataListCell key="info">
                    <strong>{flow.label}</strong>
                    <br />
                    <Label isCompact>{flow.steps.length} steps</Label>
                  </DataListCell>,
                ]}
              />
              <DataListAction id={`action-${flow.id}`} aria-label="Actions" aria-labelledby={flow.id}>
                <Button variant="plain" aria-label={`Edit ${flow.label}`} onClick={() => openEdit(flow)}><PencilAltIcon /></Button>
                <Button variant="plain" isDanger aria-label={`Delete ${flow.label}`} onClick={() => setDeleteTarget({ id: flow.id, label: flow.label })}><TrashIcon /></Button>
              </DataListAction>
            </DataListItemRow>
          </DataListItem>
        ))}
      </DataList>}

      <ConfirmDeleteModal
        isOpen={!!deleteTarget}
        itemName={deleteTarget?.label ?? ''}
        itemType="flow"
        onConfirm={() => { if (deleteTarget) { handleDelete(deleteTarget.id); setDeleteTarget(null); } }}
        onCancel={() => setDeleteTarget(null)}
      />

      {modalOpen && editFlow && (
        <Modal variant={ModalVariant.medium} isOpen onClose={() => setModalOpen(false)}>
          <ModalHeader title={isNew ? 'Add Flow' : `Edit: ${editFlow.label}`} />
          <ModalBody>
            <Form>
              {isNew && (
                <FormGroup label="ID" fieldId="flow-id">
                  <TextInput id="flow-id" value={editFlow.id} onChange={(_e, v) => setEditFlow({ ...editFlow, id: v })} placeholder="auto-generated" />
                </FormGroup>
              )}
              <FormGroup label="Label" fieldId="flow-label" isRequired>
                <TextInput id="flow-label" value={editFlow.label} onChange={(_e, v) => setEditFlow({ ...editFlow, label: v })} isRequired />
              </FormGroup>
              <FormGroup label="Description" fieldId="flow-desc">
                <TextArea id="flow-desc" value={editFlow.description ?? ''} onChange={(_e, v) => setEditFlow({ ...editFlow, description: v || undefined })} />
              </FormGroup>
              <FormGroup label="Speed" fieldId="flow-speed">
                <FormSelect id="flow-speed" value={editFlow.style?.speed ?? 'normal'} onChange={(_e, v) => setEditFlow({ ...editFlow, style: { ...editFlow.style, speed: v as FlowSpeed } })}>
                  {SPEEDS.map(s => <FormSelectOption key={s} value={s} label={s} />)}
                </FormSelect>
              </FormGroup>
              <FormGroup label="Animation" fieldId="flow-anim">
                <FormSelect id="flow-anim" value={editFlow.style?.animation ?? 'highlight'} onChange={(_e, v) => setEditFlow({ ...editFlow, style: { ...editFlow.style, animation: v as FlowAnimation } })}>
                  {ANIMATIONS.map(a => <FormSelectOption key={a} value={a} label={a} />)}
                </FormSelect>
              </FormGroup>
              <FormGroup label="Color" fieldId="flow-color">
                <TextInput id="flow-color" value={editFlow.style?.color ?? ''} onChange={(_e, v) => setEditFlow({ ...editFlow, style: { ...editFlow.style, color: v || undefined } })} placeholder="#10B981" />
              </FormGroup>

              <FormGroup label="Steps" fieldId="flow-steps">
                {editFlow.steps.map((step, idx) => (
                  <div key={idx} style={{ marginBottom: 12, padding: 8, border: '1px solid var(--pf-t--global--border--color--default)', borderRadius: 6 }}>
                    <div style={{ display: 'flex', gap: 8, marginBottom: 8, alignItems: 'center' }}>
                      <strong style={{ minWidth: 20 }}>{idx + 1}.</strong>
                      <FormSelect id={`flow-step-${idx}-edge`} aria-label={`Step ${idx + 1} edge`} value={step.edge} onChange={(_e, v) => updateStep(idx, 'edge', v)} style={{ flex: 1 }}>
                        <FormSelectOption value="" label="Select edge..." />
                        {edges.map(e => <FormSelectOption key={e.id} value={e.id} label={edgeLabel(e.id)} />)}
                      </FormSelect>
                      <Button variant="plain" aria-label={`Move step ${idx + 1} up`} isDisabled={idx === 0} onClick={() => moveStep(idx, -1)}><ArrowUpIcon /></Button>
                      <Button variant="plain" aria-label={`Move step ${idx + 1} down`} isDisabled={idx === editFlow.steps.length - 1} onClick={() => moveStep(idx, 1)}><ArrowDownIcon /></Button>
                      <Button variant="plain" isDanger aria-label={`Remove step ${idx + 1}`} onClick={() => removeStep(idx)}><TrashIcon /></Button>
                    </div>
                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr auto', gap: 8, alignItems: 'center' }}>
                      <TextInput id={`flow-step-${idx}-label`} aria-label={`Step ${idx + 1} label`} value={step.label ?? ''} onChange={(_e, v) => updateStep(idx, 'label', v)} placeholder="Step label" />
                      <TextInput id={`flow-step-${idx}-description`} aria-label={`Step ${idx + 1} description`} value={step.description ?? ''} onChange={(_e, v) => updateStep(idx, 'description', v)} placeholder="Step description" />
                      <label><input type="checkbox" aria-label={`Step ${idx + 1} parallel`} checked={step.parallel ?? false} onChange={e => updateStep(idx, 'parallel', e.target.checked)} /> Parallel</label>
                    </div>
                  </div>
                ))}
                <Button variant="link" icon={<PlusCircleIcon />} onClick={addStep} size="sm">Add Step</Button>
              </FormGroup>
              <MetadataFields idPrefix="flow" value={editFlow.metadata} onChange={metadata => setEditFlow({ ...editFlow, metadata })} />
            </Form>
          </ModalBody>
          <ModalFooter>
            <Button variant="primary" onClick={handleSave} isDisabled={!editFlow.label}>
              {isNew ? 'Add' : 'Save'}
            </Button>
            <Button variant="link" onClick={() => setModalOpen(false)}>Cancel</Button>
          </ModalFooter>
        </Modal>
      )}
    </>
  );
}
