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
} from '@patternfly/react-core';
import { PencilAltIcon, TrashIcon, PlusCircleIcon } from '@patternfly/react-icons';
import type { DiagramEdge, DiagramNode, EdgeDirection } from '../types';
import { api } from '../api';
import { ConfirmDeleteModal } from './ConfirmDeleteModal';
import { StyleFields } from './StyleFields';
import { MetadataFields } from './MetadataFields';
import { ElementEmptyState } from './ElementEmptyState';

const DIRECTIONS: EdgeDirection[] = ['one-way', 'two-way', 'none'];

interface Props {
  diagramName: string;
  onChange: () => void;
  onError: (msg: string) => void;
}

function emptyEdge(): DiagramEdge {
  return { id: '', from: '', to: '', direction: 'one-way', metadata: {} };
}

export function EdgesTab({ diagramName, onChange, onError }: Props) {
  const [edges, setEdges] = useState<DiagramEdge[]>([]);
  const [nodes, setNodes] = useState<DiagramNode[]>([]);
  const [editEdge, setEditEdge] = useState<DiagramEdge | null>(null);
  const [isNew, setIsNew] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; label: string } | null>(null);

  const load = useCallback(async () => {
    try {
      const [e, n] = await Promise.all([
        api.listEdges(diagramName),
        api.listNodes(diagramName),
      ]);
      setEdges(e);
      setNodes(n);
    } catch (e) {
      onError(`${e}`);
    }
  }, [diagramName, onError]);

  useEffect(() => { load(); }, [load]);

  const openAdd = () => {
    setEditEdge(emptyEdge());
    setIsNew(true);
    setModalOpen(true);
  };

  const openEdit = (edge: DiagramEdge) => {
    setEditEdge({ ...edge });
    setIsNew(false);
    setModalOpen(true);
  };

  const handleSave = async () => {
    if (!editEdge) return;
    try {
      if (isNew) {
        const id = editEdge.id || `${editEdge.from}-to-${editEdge.to}`;
        await api.addEdge(diagramName, { ...editEdge, id });
      } else {
        await api.updateEdge(diagramName, editEdge.id, editEdge);
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
      await api.deleteEdge(diagramName, id);
      await load();
      onChange();
    } catch (e) {
      onError(`${e}`);
    }
  };

  const nodeLabel = (id: string) => nodes.find(n => n.id === id)?.label ?? id;

  return (
    <>
      {edges.length > 0 && <Button variant="link" icon={<PlusCircleIcon />} onClick={openAdd} style={{ marginBottom: '8px' }}>
        Add Edge
      </Button>}
      {edges.length === 0 ? <ElementEmptyState title="No edges yet" description="Add an edge to connect nodes in the architecture." actionLabel="Add edge" onCreate={openAdd} /> : <DataList aria-label="Edges" isCompact>
        {edges.map(edge => (
          <DataListItem key={edge.id} id={edge.id}>
            <DataListItemRow>
              <DataListItemCells
                dataListCells={[
                  <DataListCell key="info">
                    <strong>{nodeLabel(edge.from)}</strong>
                    {' → '}
                    <strong>{nodeLabel(edge.to)}</strong>
                    {edge.label && <><br /><small>{edge.label}</small></>}
                  </DataListCell>,
                ]}
              />
              <DataListAction id={`action-${edge.id}`} aria-label="Actions" aria-labelledby={edge.id}>
                <Button variant="plain" aria-label={`Edit edge ${edge.id}`} onClick={() => openEdit(edge)}><PencilAltIcon /></Button>
                <Button variant="plain" isDanger aria-label={`Delete edge ${edge.id}`} onClick={() => setDeleteTarget({ id: edge.id, label: `${nodeLabel(edge.from)} → ${nodeLabel(edge.to)}` })}><TrashIcon /></Button>
              </DataListAction>
            </DataListItemRow>
          </DataListItem>
        ))}
      </DataList>}

      <ConfirmDeleteModal
        isOpen={!!deleteTarget}
        itemName={deleteTarget?.label ?? ''}
        itemType="edge"
        onConfirm={() => { if (deleteTarget) { handleDelete(deleteTarget.id); setDeleteTarget(null); } }}
        onCancel={() => setDeleteTarget(null)}
      />

      {modalOpen && editEdge && (
        <Modal variant={ModalVariant.small} isOpen onClose={() => setModalOpen(false)}>
          <ModalHeader title={isNew ? 'Add Edge' : `Edit: ${editEdge.id}`} />
          <ModalBody>
            <Form>
              {isNew && (
                <FormGroup label="ID" fieldId="edge-id">
                  <TextInput id="edge-id" value={editEdge.id} onChange={(_e, v) => setEditEdge({ ...editEdge, id: v })} placeholder="auto-generated" />
                </FormGroup>
              )}
              <FormGroup label="From" fieldId="edge-from" isRequired>
                <FormSelect id="edge-from" value={editEdge.from} onChange={(_e, v) => setEditEdge({ ...editEdge, from: v })}>
                  <FormSelectOption value="" label="Select node..." />
                  {nodes.map(n => <FormSelectOption key={n.id} value={n.id} label={n.label} />)}
                </FormSelect>
              </FormGroup>
              <FormGroup label="To" fieldId="edge-to" isRequired>
                <FormSelect id="edge-to" value={editEdge.to} onChange={(_e, v) => setEditEdge({ ...editEdge, to: v })}>
                  <FormSelectOption value="" label="Select node..." />
                  {nodes.map(n => <FormSelectOption key={n.id} value={n.id} label={n.label} />)}
                </FormSelect>
              </FormGroup>
              <FormGroup label="Label" fieldId="edge-label">
                <TextInput id="edge-label" value={editEdge.label ?? ''} onChange={(_e, v) => setEditEdge({ ...editEdge, label: v || undefined })} />
              </FormGroup>
              <FormGroup label="Direction" fieldId="edge-direction">
                <FormSelect id="edge-direction" value={editEdge.direction ?? 'one-way'} onChange={(_e, v) => setEditEdge({ ...editEdge, direction: v as EdgeDirection })}>
                  {DIRECTIONS.map(d => <FormSelectOption key={d} value={d} label={d} />)}
                </FormSelect>
              </FormGroup>
              <StyleFields idPrefix="edge-style" value={editEdge.style} onChange={style => setEditEdge({ ...editEdge, style })} />
              <MetadataFields idPrefix="edge" value={editEdge.metadata} onChange={metadata => setEditEdge({ ...editEdge, metadata })} />
            </Form>
          </ModalBody>
          <ModalFooter>
            <Button variant="primary" onClick={handleSave} isDisabled={!editEdge.from || !editEdge.to}>
              {isNew ? 'Add' : 'Save'}
            </Button>
            <Button variant="link" onClick={() => setModalOpen(false)}>Cancel</Button>
          </ModalFooter>
        </Modal>
      )}
    </>
  );
}
