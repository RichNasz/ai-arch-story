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
} from '@patternfly/react-core';
import { PencilAltIcon, TrashIcon, PlusCircleIcon } from '@patternfly/react-icons';
import type { DiagramNode, ResolvedType } from '../types';
import { api } from '../api';
import { ConfirmDeleteModal } from './ConfirmDeleteModal';
import { StyleFields } from './StyleFields';
import { MetadataFields } from './MetadataFields';

interface Props {
  diagramName: string;
  onChange: () => void;
  onError: (msg: string) => void;
}

function emptyNode(): DiagramNode {
  return { id: '', label: '', type: 'service', metadata: {} };
}

function accentToLabelColor(color: string): 'blue' | 'purple' | 'orange' | 'teal' | 'grey' | 'red' | 'green' {
  const map: Record<string, 'blue' | 'purple' | 'orange' | 'teal' | 'grey' | 'red' | 'green'> = {
    '#3B82F6': 'blue', '#8B5CF6': 'purple', '#F59E0B': 'orange', '#06B6D4': 'teal',
    '#10B981': 'green', '#6B7280': 'grey', '#EC4899': 'red', '#14B8A6': 'teal',
    '#A78BFA': 'purple', '#94A3B8': 'grey',
  };
  return map[color] || 'grey';
}

export function NodesTab({ diagramName, onChange, onError }: Props) {
  const [nodes, setNodes] = useState<DiagramNode[]>([]);
  const [availableTypes, setAvailableTypes] = useState<ResolvedType[]>([]);
  const [editNode, setEditNode] = useState<DiagramNode | null>(null);
  const [isNew, setIsNew] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; label: string } | null>(null);

  const load = useCallback(async () => {
    try {
      const [nodeList, registry] = await Promise.all([
        api.listNodes(diagramName),
        api.getResolvedTypes(diagramName),
      ]);
      setNodes(nodeList);
      setAvailableTypes(Object.values(registry.types));
    } catch (e) {
      onError(`${e}`);
    }
  }, [diagramName, onError]);

  useEffect(() => { load(); }, [load]);

  const openAdd = () => {
    setEditNode(emptyNode());
    setIsNew(true);
    setModalOpen(true);
  };

  const openEdit = (node: DiagramNode) => {
    setEditNode({ ...node });
    setIsNew(false);
    setModalOpen(true);
  };

  const handleSave = async () => {
    if (!editNode) return;
    try {
      if (isNew) {
        const id = editNode.id || editNode.label.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
        await api.addNode(diagramName, { ...editNode, id });
      } else {
        await api.updateNode(diagramName, editNode.id, editNode);
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
      await api.deleteNode(diagramName, id);
      await load();
      onChange();
    } catch (e) {
      onError(`${e}`);
    }
  };

  const update = (field: string, value: string) => {
    if (!editNode) return;
    setEditNode({ ...editNode, [field]: value || undefined });
  };

  return (
    <>
      <Button variant="link" icon={<PlusCircleIcon />} onClick={openAdd} style={{ marginBottom: '8px' }}>
        Add Node
      </Button>
      <DataList aria-label="Nodes" isCompact>
        {nodes.map(node => (
          <DataListItem key={node.id} id={node.id}>
            <DataListItemRow>
              <DataListItemCells
                dataListCells={[
                  <DataListCell key="label">
                    <strong>{node.label}</strong>
                    <br />
                    <Label color={accentToLabelColor(availableTypes.find(t => t.key === (node.type ?? 'generic'))?.accent_color ?? '#94A3B8')} isCompact>
                      {availableTypes.find(t => t.key === (node.type ?? 'generic'))?.label ?? (node.type ?? 'generic')}
                    </Label>
                  </DataListCell>,
                ]}
              />
              <DataListAction
                id={`action-${node.id}`}
                aria-label="Actions"
                aria-labelledby={node.id}
              >
                <Button variant="plain" aria-label={`Edit ${node.label}`} onClick={() => openEdit(node)}>
                  <PencilAltIcon />
                </Button>
                <Button variant="plain" isDanger aria-label={`Delete ${node.label}`} onClick={() => setDeleteTarget({ id: node.id, label: node.label })}>
                  <TrashIcon />
                </Button>
              </DataListAction>
            </DataListItemRow>
          </DataListItem>
        ))}
      </DataList>

      <ConfirmDeleteModal
        isOpen={!!deleteTarget}
        itemName={deleteTarget?.label ?? ''}
        itemType="node"
        onConfirm={() => { if (deleteTarget) { handleDelete(deleteTarget.id); setDeleteTarget(null); } }}
        onCancel={() => setDeleteTarget(null)}
      />

      {modalOpen && editNode && (
        <Modal
          variant={ModalVariant.small}
          isOpen
          onClose={() => setModalOpen(false)}
        >
          <ModalHeader title={isNew ? 'Add Node' : `Edit: ${editNode.label}`} />
          <ModalBody>
            <Form>
              {isNew && (
                <FormGroup label="ID" fieldId="node-id">
                  <TextInput
                    id="node-id"
                    value={editNode.id}
                    onChange={(_e, v) => update('id', v)}
                    placeholder="auto-generated from label"
                  />
                </FormGroup>
              )}
              <FormGroup label="Label" fieldId="node-label" isRequired>
                <TextInput
                  id="node-label"
                  value={editNode.label}
                  onChange={(_e, v) => update('label', v)}
                  isRequired
                />
              </FormGroup>
              <FormGroup label="Type" fieldId="node-type">
                <FormSelect
                  id="node-type"
                  value={editNode.type ?? 'generic'}
                  onChange={(_e, v) => update('type', v)}
                >
                  {availableTypes.map(t => (
                    <FormSelectOption key={t.key} value={t.key} label={t.label} />
                  ))}
                </FormSelect>
              </FormGroup>
              <FormGroup label="Icon" fieldId="node-icon">
                <TextInput
                  id="node-icon"
                  value={editNode.icon ?? ''}
                  onChange={(_e, v) => update('icon', v)}
                  placeholder="Optional icon identifier"
                />
              </FormGroup>
              <StyleFields idPrefix="node-style" value={editNode.style} includeShape onChange={style => setEditNode({ ...editNode, style })} />
              <MetadataFields idPrefix="node" value={editNode.metadata} onChange={metadata => setEditNode({ ...editNode, metadata })} />
            </Form>
          </ModalBody>
          <ModalFooter>
            <Button variant="primary" onClick={handleSave} isDisabled={!editNode.label}>
              {isNew ? 'Add' : 'Save'}
            </Button>
            <Button variant="link" onClick={() => setModalOpen(false)}>Cancel</Button>
          </ModalFooter>
        </Modal>
      )}
    </>
  );
}
