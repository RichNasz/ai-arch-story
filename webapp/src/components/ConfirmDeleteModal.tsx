import {
  Modal,
  ModalVariant,
  ModalBody,
  ModalFooter,
  ModalHeader,
  Button,
} from '@patternfly/react-core';

interface Props {
  isOpen: boolean;
  itemName: string;
  itemType: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDeleteModal({ isOpen, itemName, itemType, onConfirm, onCancel }: Props) {
  return (
    <Modal variant={ModalVariant.small} isOpen={isOpen} onClose={onCancel}>
      <ModalHeader title={`Delete ${itemType}?`} titleIconVariant="warning" />
      <ModalBody>
        <strong>{itemName}</strong> will be permanently deleted. This action cannot be undone.
      </ModalBody>
      <ModalFooter>
        <Button variant="danger" onClick={onConfirm}>Delete</Button>
        <Button variant="link" onClick={onCancel}>Cancel</Button>
      </ModalFooter>
    </Modal>
  );
}
