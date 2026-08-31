import { useEffect, useState } from 'react';
import {
  Button, Form, FormGroup, Modal, ModalBody, ModalFooter, ModalHeader,
  ModalVariant, TextInput,
} from '@patternfly/react-core';

interface Props {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (name: string, title: string) => Promise<void>;
}

function slugify(value: string) {
  return value.toLowerCase().trim().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
}

export function CreateDiagramModal({ isOpen, onClose, onCreate }: Props) {
  const [title, setTitle] = useState('');
  const [name, setName] = useState('');
  const [nameEdited, setNameEdited] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!isOpen) {
      setTitle(''); setName(''); setNameEdited(false); setSaving(false);
    }
  }, [isOpen]);

  const submit = async () => {
    setSaving(true);
    try { await onCreate(name, title); } finally { setSaving(false); }
  };

  return (
    <Modal variant={ModalVariant.small} isOpen={isOpen} onClose={onClose}>
      <ModalHeader title="Create diagram" />
      <ModalBody>
        <Form>
          <FormGroup label="Title" fieldId="diagram-title" isRequired>
            <TextInput id="diagram-title" value={title} isRequired onChange={(_e, value) => {
              setTitle(value);
              if (!nameEdited) setName(slugify(value));
            }} />
          </FormGroup>
          <FormGroup label="Name" fieldId="diagram-name" isRequired>
            <TextInput id="diagram-name" value={name} isRequired onChange={(_e, value) => {
              setName(slugify(value)); setNameEdited(true);
            }} />
          </FormGroup>
        </Form>
      </ModalBody>
      <ModalFooter>
        <Button variant="primary" onClick={submit} isLoading={saving} isDisabled={saving || !title.trim() || !name}>Create</Button>
        <Button variant="link" onClick={onClose}>Cancel</Button>
      </ModalFooter>
    </Modal>
  );
}
