import { useState, useEffect, useCallback } from 'react';
import {
  Page,
  Masthead,
  MastheadMain,
  MastheadBrand,
  MastheadContent,
  PageSection,
  Select,
  SelectOption,
  MenuToggle,
  MenuToggleElement,
  Button,
  Alert,
  AlertActionCloseButton,
  Toolbar,
  ToolbarContent,
  ToolbarItem,
  Split,
  SplitItem,
  EmptyState,
  EmptyStateBody,
  EmptyStateFooter,
  EmptyStateActions,
} from '@patternfly/react-core';
import { PlusCircleIcon } from '@patternfly/react-icons';
import type { DiagramListEntry } from './types';
import { api } from './api';
import { EditorSidebar } from './components/EditorSidebar';
import { PreviewPane } from './components/PreviewPane';
import { CreateDiagramModal } from './components/CreateDiagramModal';
import { StatusBar } from './components/StatusBar';

export function App() {
  const [diagrams, setDiagrams] = useState<DiagramListEntry[]>([]);
  const [selectedDiagram, setSelectedDiagram] = useState<string | null>(null);
  const [selectOpen, setSelectOpen] = useState(false);
  const [alert, setAlert] = useState<{ variant: 'success' | 'danger' | 'info'; title: string } | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [createOpen, setCreateOpen] = useState(false);
  const [validation, setValidation] = useState<'unknown' | 'valid' | 'invalid'>('unknown');
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [outputUrl, setOutputUrl] = useState<string>();

  const loadDiagrams = useCallback(async () => {
    try {
      const list = await api.listDiagrams();
      setDiagrams(list);
      if (list.length > 0 && !selectedDiagram) {
        setSelectedDiagram(list[0].name);
      }
    } catch (e) {
      setAlert({ variant: 'danger', title: `Failed to load diagrams: ${e}` });
    }
  }, [selectedDiagram]);

  useEffect(() => { loadDiagrams(); }, [loadDiagrams]);

  const handleRender = async () => {
    if (!selectedDiagram) return;
    try {
      const result = await api.render(selectedDiagram);
      setAlert({ variant: 'success', title: `Rendered: ${result.outputPath}` });
      setOutputUrl(api.getPreviewUrl(selectedDiagram));
      setDiagrams(current => current.map(d => d.name === selectedDiagram ? { ...d, hasOutput: true } : d));
      setRefreshKey(k => k + 1);
    } catch (e) {
      setAlert({ variant: 'danger', title: `Render failed: ${e}` });
    }
  };

  const handleValidate = async () => {
    if (!selectedDiagram) return;
    try {
      await api.validate(selectedDiagram);
      setValidation('valid');
      setAlert({ variant: 'success', title: 'Diagram is valid' });
    } catch (e) {
      setValidation('invalid');
      setAlert({ variant: 'danger', title: `Validation failed: ${e}` });
    }
  };

  const handleChange = async () => {
    if (!selectedDiagram) return;
    setLastSaved(new Date());
    setValidation('unknown');
    try {
      await api.render(selectedDiagram);
      setDiagrams(current => current.map(d => d.name === selectedDiagram ? { ...d, hasOutput: true } : d));
      setOutputUrl(api.getPreviewUrl(selectedDiagram));
      setRefreshKey(k => k + 1);
    } catch (e) {
      setAlert({ variant: 'danger', title: `Saved, but preview refresh failed: ${e}` });
    }
  };

  const handleCreate = async (name: string, title: string) => {
    try {
      await api.createDiagram(name, title);
      setDiagrams(current => [...current, { name, title, hasOutput: false }].sort((a, b) => a.name.localeCompare(b.name)));
      setSelectedDiagram(name);
      setRefreshKey(0);
      setCreateOpen(false);
      setAlert({ variant: 'success', title: `Created ${title}` });
    } catch (e) {
      setAlert({ variant: 'danger', title: `Create failed: ${e}` });
      throw e;
    }
  };

  const handleSidebarError = useCallback((msg: string) => {
    setAlert({ variant: 'danger', title: msg });
  }, []);

  const selectToggle = (toggleRef: React.Ref<MenuToggleElement>) => (
    <MenuToggle
      ref={toggleRef}
      onClick={() => setSelectOpen(!selectOpen)}
      isExpanded={selectOpen}
      style={{ minWidth: '250px' }}
    >
      {selectedDiagram
        ? diagrams.find(d => d.name === selectedDiagram)?.title ?? selectedDiagram
        : 'Select diagram'}
    </MenuToggle>
  );

  return (
    <Page
      masthead={
        <Masthead>
          <MastheadMain>
            <MastheadBrand>AI Arch Story</MastheadBrand>
          </MastheadMain>
          <MastheadContent>
            <Toolbar>
              <ToolbarContent>
                <ToolbarItem>
                  <Button variant="secondary" icon={<PlusCircleIcon />} onClick={() => setCreateOpen(true)}>
                    New diagram
                  </Button>
                </ToolbarItem>
                <ToolbarItem>
                  <Select
                    isOpen={selectOpen}
                    selected={selectedDiagram ?? undefined}
                    onSelect={(_e, value) => {
                      setSelectedDiagram(value as string);
                      setRefreshKey(0);
                      setSelectOpen(false);
                    }}
                    onOpenChange={setSelectOpen}
                    toggle={selectToggle}
                  >
                    {diagrams.map(d => (
                      <SelectOption key={d.name} value={d.name}>
                        {d.title}
                      </SelectOption>
                    ))}
                  </Select>
                </ToolbarItem>
                <ToolbarItem>
                  <Button variant="secondary" onClick={handleValidate} isDisabled={!selectedDiagram}>
                    Validate
                  </Button>
                </ToolbarItem>
                <ToolbarItem>
                  <Button variant="primary" onClick={handleRender} isDisabled={!selectedDiagram}>
                    Re-layout
                  </Button>
                </ToolbarItem>
              </ToolbarContent>
            </Toolbar>
          </MastheadContent>
        </Masthead>
      }
    >
      {/* Tighter padding than default PageSection for the dismissable alert banner */}
      {alert && (
        <PageSection padding={{ default: 'noPadding' }} style={{ padding: '8px 16px' }}>
          <Alert
            variant={alert.variant}
            title={alert.title}
            actionClose={<AlertActionCloseButton onClose={() => setAlert(null)} />}
            timeout={5000}
            onTimeout={() => setAlert(null)}
          />
        </PageSection>
      )}
      <PageSection isFilled padding={{ default: 'noPadding' }} style={{ height: 'calc(100vh - 112px)' }}>
        {selectedDiagram ? (
          <Split hasGutter style={{ height: '100%' }}>
            <SplitItem style={{ width: '380px', overflow: 'auto', borderRight: '1px solid var(--pf-t--global--border--color--default)' }}>
              <EditorSidebar
                diagramName={selectedDiagram}
                onChange={handleChange}
                onError={handleSidebarError}
              />
            </SplitItem>
            <SplitItem isFilled style={{ position: 'relative' }}>
              <PreviewPane
                key={selectedDiagram}
                diagramName={selectedDiagram}
                refreshKey={refreshKey}
                hasOutput={diagrams.find(d => d.name === selectedDiagram)?.hasOutput ?? false}
              />
            </SplitItem>
          </Split>
        ) : (
          <EmptyState headingLevel="h3" titleText="No diagrams yet" icon={PlusCircleIcon}>
            <EmptyStateBody>
              Create your first architecture diagram to get started.
            </EmptyStateBody>
            <EmptyStateFooter>
              <EmptyStateActions>
                <Button variant="primary" onClick={() => setCreateOpen(true)}>Create diagram</Button>
              </EmptyStateActions>
            </EmptyStateFooter>
          </EmptyState>
        )}
      </PageSection>
      <StatusBar validation={validation} lastSaved={lastSaved} outputUrl={outputUrl} />
      <CreateDiagramModal isOpen={createOpen} onClose={() => setCreateOpen(false)} onCreate={handleCreate} />
    </Page>
  );
}
