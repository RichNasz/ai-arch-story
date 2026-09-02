import { useState } from 'react';
import { Tabs, Tab, TabTitleText } from '@patternfly/react-core';
import { NodesTab } from './NodesTab';
import { EdgesTab } from './EdgesTab';
import { FlowsTab } from './FlowsTab';
import { GroupsTab } from './GroupsTab';
import { TypesTab } from './TypesTab';
import { BrandingTab } from './BrandingTab';

interface Props {
  diagramName: string;
  onChange: () => void;
  onError: (msg: string) => void;
}

export function EditorSidebar({ diagramName, onChange, onError }: Props) {
  const [activeTab, setActiveTab] = useState<string | number>('nodes');

  return (
    <div style={{ padding: '8px' }}>
      <Tabs
        activeKey={activeTab}
        onSelect={(_e, key) => setActiveTab(key)}
        isVertical
        style={{ marginBottom: '16px' }}
      >
        <Tab eventKey="nodes" title={<TabTitleText>Nodes</TabTitleText>}>
          <NodesTab diagramName={diagramName} onChange={onChange} onError={onError} />
        </Tab>
        <Tab eventKey="edges" title={<TabTitleText>Edges</TabTitleText>}>
          <EdgesTab diagramName={diagramName} onChange={onChange} onError={onError} />
        </Tab>
        <Tab eventKey="flows" title={<TabTitleText>Flows</TabTitleText>}>
          <FlowsTab diagramName={diagramName} onChange={onChange} onError={onError} />
        </Tab>
        <Tab eventKey="groups" title={<TabTitleText>Groups</TabTitleText>}>
          <GroupsTab diagramName={diagramName} onChange={onChange} onError={onError} />
        </Tab>
        <Tab eventKey="types" title={<TabTitleText>Types</TabTitleText>}>
          <TypesTab diagramName={diagramName} onChange={onChange} onError={onError} />
        </Tab>
        <Tab eventKey="branding" title={<TabTitleText>Branding</TabTitleText>}>
          <BrandingTab onChange={onChange} onError={onError} />
        </Tab>
      </Tabs>
    </div>
  );
}
