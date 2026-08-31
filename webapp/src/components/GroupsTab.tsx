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
  Modal,
  ModalVariant,
  ModalBody,
  ModalFooter,
  ModalHeader,
  Label,
  Select,
  SelectOption,
  MenuToggle,
  MenuToggleElement,
} from '@patternfly/react-core';
import { PencilAltIcon, TrashIcon, PlusCircleIcon } from '@patternfly/react-icons';
import type { DiagramGroup, DiagramNode } from '../types';
import { api } from '../api';
import { ConfirmDeleteModal } from './ConfirmDeleteModal';
import { StyleFields } from './StyleFields';
import { MetadataFields } from './MetadataFields';

interface Props {
  diagramName: string;
  onChange: () => void;
  onError: (msg: string) => void;
}

function emptyGroup(): DiagramGroup {
  return { id: '', label: '', nodes: [], groups: [], metadata: {} };
}

export function GroupsTab({ diagramName, onChange, onError }: Props) {
  const [groups, setGroups] = useState<DiagramGroup[]>([]);
  const [nodes, setNodes] = useState<DiagramNode[]>([]);
  const [editGroup, setEditGroup] = useState<DiagramGroup | null>(null);
  const [isNew, setIsNew] = useState(false);
  const [modalOpen, setModalOpen] = useState(false);
  const [nodeSelectOpen, setNodeSelectOpen] = useState(false);
  const [groupSelectOpen, setGroupSelectOpen] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; label: string } | null>(null);

  const load = useCallback(async () => {
    try {
      const [g, n] = await Promise.all([
        api.listGroups(diagramName),
        api.listNodes(diagramName),
      ]);
      setGroups(g);
      setNodes(n);
    } catch (e) {
      onError(`${e}`);
    }
  }, [diagramName, onError]);

  useEffect(() => { load(); }, [load]);

  const openAdd = () => {
    setEditGroup(emptyGroup());
    setIsNew(true);
    setModalOpen(true);
  };

  const openEdit = (group: DiagramGroup) => {
    setEditGroup({ ...group, nodes: [...group.nodes], groups: [...group.groups] });
    setIsNew(false);
    setModalOpen(true);
  };

  const handleSave = async () => {
    if (!editGroup) return;
    try {
      if (isNew) {
        const id = editGroup.id || editGroup.label.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
        await api.addGroup(diagramName, { ...editGroup, id });
      } else {
        await api.updateGroup(diagramName, editGroup.id, editGroup);
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
      await api.deleteGroup(diagramName, id);
      await load();
      onChange();
    } catch (e) {
      onError(`${e}`);
    }
  };

  const toggleNode = (nodeId: string) => {
    if (!editGroup) return;
    const selected = editGroup.nodes.includes(nodeId)
      ? editGroup.nodes.filter(n => n !== nodeId)
      : [...editGroup.nodes, nodeId];
    setEditGroup({ ...editGroup, nodes: selected });
  };

  const toggleGroup = (groupId: string) => {
    if (!editGroup) return;
    setEditGroup({ ...editGroup, groups: editGroup.groups.includes(groupId) ? editGroup.groups.filter(g => g !== groupId) : [...editGroup.groups, groupId] });
  };

  const nodeLabel = (id: string) => nodes.find(n => n.id === id)?.label ?? id;

  const nodeSelectToggle = (toggleRef: React.Ref<MenuToggleElement>) => (
    <MenuToggle
      ref={toggleRef}
      onClick={() => setNodeSelectOpen(!nodeSelectOpen)}
      isExpanded={nodeSelectOpen}
      style={{ width: '100%' }}
    >
      {editGroup && editGroup.nodes.length > 0
        ? `${editGroup.nodes.length} nodes selected`
        : 'Select nodes...'}
    </MenuToggle>
  );

  return (
    <>
      <Button variant="link" icon={<PlusCircleIcon />} onClick={openAdd} style={{ marginBottom: '8px' }}>
        Add Group
      </Button>
      <DataList aria-label="Groups" isCompact>
        {groups.map(group => (
          <DataListItem key={group.id} id={group.id}>
            <DataListItemRow>
              <DataListItemCells
                dataListCells={[
                  <DataListCell key="info">
                    <strong>{group.label}</strong>
                    <br />
                    <Label isCompact>{group.nodes.length} nodes</Label>
                  </DataListCell>,
                ]}
              />
              <DataListAction id={`action-${group.id}`} aria-label="Actions" aria-labelledby={group.id}>
                <Button variant="plain" aria-label={`Edit ${group.label}`} onClick={() => openEdit(group)}><PencilAltIcon /></Button>
                <Button variant="plain" isDanger aria-label={`Delete ${group.label}`} onClick={() => setDeleteTarget({ id: group.id, label: group.label })}><TrashIcon /></Button>
              </DataListAction>
            </DataListItemRow>
          </DataListItem>
        ))}
      </DataList>

      <ConfirmDeleteModal
        isOpen={!!deleteTarget}
        itemName={deleteTarget?.label ?? ''}
        itemType="group"
        onConfirm={() => { if (deleteTarget) { handleDelete(deleteTarget.id); setDeleteTarget(null); } }}
        onCancel={() => setDeleteTarget(null)}
      />

      {modalOpen && editGroup && (
        <Modal
          variant={ModalVariant.small}
          isOpen
          onClose={() => setModalOpen(false)}
          focusTrapId="groups-modal-focus-trap"
          onEscapePress={(event) => {
            if (nodeSelectOpen) {
              setNodeSelectOpen(false);
              event.stopPropagation();
            } else {
              setModalOpen(false);
            }
          }}
        >
          <ModalHeader title={isNew ? 'Add Group' : `Edit: ${editGroup.label}`} />
          <ModalBody>
            <Form>
              {isNew && (
                <FormGroup label="ID" fieldId="group-id">
                  <TextInput id="group-id" value={editGroup.id} onChange={(_e, v) => setEditGroup({ ...editGroup, id: v })} placeholder="auto-generated" />
                </FormGroup>
              )}
              <FormGroup label="Label" fieldId="group-label" isRequired>
                <TextInput id="group-label" value={editGroup.label} onChange={(_e, v) => setEditGroup({ ...editGroup, label: v })} isRequired />
              </FormGroup>
              <FormGroup label="Nodes" fieldId="group-nodes">
                <Select
                  isOpen={nodeSelectOpen}
                  selected={editGroup.nodes}
                  onSelect={(_e, value) => toggleNode(value as string)}
                  onOpenChange={setNodeSelectOpen}
                  toggle={nodeSelectToggle}
                  popperProps={{ appendTo: () => document.getElementById('groups-modal-focus-trap')! }}
                >
                  {nodes.map(n => (
                    <SelectOption
                      key={n.id}
                      value={n.id}
                      hasCheckbox
                      isSelected={editGroup.nodes.includes(n.id)}
                    >
                      {n.label}
                    </SelectOption>
                  ))}
                </Select>
                {editGroup.nodes.length > 0 && (
                  <div style={{ marginTop: '8px', display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                    {editGroup.nodes.map(nid => (
                      <Label key={nid} isCompact onClose={() => toggleNode(nid)}>
                        {nodeLabel(nid)}
                      </Label>
                    ))}
                  </div>
                )}
              </FormGroup>
              <FormGroup label="Nested groups" fieldId="group-groups">
                <Select
                  isOpen={groupSelectOpen}
                  selected={editGroup.groups}
                  onSelect={(_e, value) => toggleGroup(value as string)}
                  onOpenChange={setGroupSelectOpen}
                  toggle={toggleRef => <MenuToggle ref={toggleRef} onClick={() => setGroupSelectOpen(!groupSelectOpen)} isExpanded={groupSelectOpen} style={{ width: '100%' }}>{editGroup.groups.length ? `${editGroup.groups.length} groups selected` : 'Select groups...'}</MenuToggle>}
                  popperProps={{ appendTo: () => document.getElementById('groups-modal-focus-trap')! }}
                >
                  {groups.filter(g => g.id !== editGroup.id).map(g => <SelectOption key={g.id} value={g.id} hasCheckbox isSelected={editGroup.groups.includes(g.id)}>{g.label}</SelectOption>)}
                </Select>
              </FormGroup>
              <StyleFields idPrefix="group-style" value={editGroup.style} onChange={style => setEditGroup({ ...editGroup, style })} />
              <MetadataFields idPrefix="group" value={editGroup.metadata} onChange={metadata => setEditGroup({ ...editGroup, metadata })} />
            </Form>
          </ModalBody>
          <ModalFooter>
            <Button variant="primary" onClick={handleSave} isDisabled={!editGroup.label}>
              {isNew ? 'Add' : 'Save'}
            </Button>
            <Button variant="link" onClick={() => setModalOpen(false)}>Cancel</Button>
          </ModalFooter>
        </Modal>
      )}
    </>
  );
}
