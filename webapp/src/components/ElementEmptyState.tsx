import { Button, EmptyState, EmptyStateActions, EmptyStateBody, EmptyStateFooter } from '@patternfly/react-core';
import { PlusCircleIcon } from '@patternfly/react-icons';

interface Props {
  title: string;
  description: string;
  actionLabel: string;
  onCreate: () => void;
}

export function ElementEmptyState({ title, description, actionLabel, onCreate }: Props) {
  return (
    <EmptyState headingLevel="h3" titleText={title} icon={PlusCircleIcon}>
      <EmptyStateBody>{description}</EmptyStateBody>
      <EmptyStateFooter>
        <EmptyStateActions><Button variant="primary" onClick={onCreate}>{actionLabel}</Button></EmptyStateActions>
      </EmptyStateFooter>
    </EmptyState>
  );
}
