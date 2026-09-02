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
  AlertGroup,
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
  const [isRendering, setIsRendering] = useState(false);
  const [isValidating, setIsValidating] = useState(false);
  const [previewStale, setPreviewStale] = useState(false);
  const [isExporting, setIsExporting] = useState(false);

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

  const renderDiagram = async () => {
    if (!selectedDiagram) return;
    const result = await api.render(selectedDiagram);
    const previewUrl = api.getPreviewUrl(selectedDiagram);
    setOutputUrl(previewUrl);
    setDiagrams(current => current.map(d => d.name === selectedDiagram ? { ...d, hasOutput: true } : d));
    setRefreshKey(k => k + 1);
    setPreviewStale(false);
    return { result, previewUrl };
  };

  const handleRender = async () => {
    if (!selectedDiagram) return false;
    setIsRendering(true);
    try {
      const rendered = await renderDiagram();
      if (!rendered) return false;
      setAlert({ variant: 'success', title: `Rendered: ${rendered.result.outputPath}` });
      return true;
    } catch (e) {
      setAlert({ variant: 'danger', title: `Render failed: ${e}` });
      return false;
    } finally {
      setIsRendering(false);
    }
  };

  const handleExport = async () => {
    if (!selectedDiagram) return;
    setIsExporting(true);
    try {
      const rendered = await renderDiagram();
      if (!rendered) return;
      const download = document.createElement('a');
      download.href = rendered.previewUrl;
      download.download = `${selectedDiagram}.html`;
      document.body.appendChild(download);
      download.click();
      download.remove();
      setAlert({ variant: 'success', title: `Exported: ${rendered.result.outputPath}` });
    } catch (e) {
      setAlert({ variant: 'danger', title: `Render failed: ${e}` });
    } finally {
      setIsExporting(false);
    }
  };

  const handleValidate = async () => {
    if (!selectedDiagram) return;
    setIsValidating(true);
    try {
      await api.validate(selectedDiagram);
      setValidation('valid');
      setAlert({ variant: 'success', title: 'Diagram is valid' });
    } catch (e) {
      setValidation('invalid');
      setAlert({ variant: 'danger', title: `Validation failed: ${e}` });
    } finally { setIsValidating(false); }
  };

  const handleChange = async () => {
    if (!selectedDiagram) return;
    setLastSaved(new Date());
    setValidation('unknown');
    setPreviewStale(true);
  };

  const handleCreate = async (name: string, title: string) => {
    try {
      await api.createDiagram(name, title);
      setDiagrams(current => [...current, { name, title, hasOutput: false }].sort((a, b) => a.name.localeCompare(b.name)));
      setSelectedDiagram(name);
      setRefreshKey(0);
      setPreviewStale(false);
      setOutputUrl(undefined);
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
                      setPreviewStale(false);
                      setOutputUrl(undefined);
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
                  <Button variant="secondary" onClick={handleValidate} isDisabled={!selectedDiagram || isValidating} isLoading={isValidating}>
                    Validate
                  </Button>
                </ToolbarItem>
                <ToolbarItem>
                  <Button variant="secondary" onClick={handleRender} isDisabled={!selectedDiagram || isRendering || isExporting} isLoading={isRendering}>
                    Re-layout
                  </Button>
                </ToolbarItem>
                <ToolbarItem>
                  <Button variant="primary" onClick={handleExport} isDisabled={!selectedDiagram || isExporting} isLoading={isExporting}>
                    Export HTML
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
        <AlertGroup isToast isLiveRegion aria-label="Editor notifications">
          <Alert
            variant={alert.variant}
            title={alert.title}
            actionClose={<AlertActionCloseButton onClose={() => setAlert(null)} />}
            timeout={5000}
            onTimeout={() => setAlert(null)}
          />
        </AlertGroup>
      )}
      <PageSection isFilled hasBodyWrapper={false} padding={{ default: 'noPadding' }} style={{ height: 'calc(100vh - 112px)', minHeight: 0 }}>
        {selectedDiagram ? (
            <Split hasGutter style={{ height: '100%', minHeight: 0 }}>
            <SplitItem style={{ width: '380px', overflow: 'auto', borderRight: '1px solid var(--pf-t--global--border--color--default)' }}>
              <EditorSidebar
                diagramName={selectedDiagram}
                onChange={handleChange}
                onError={handleSidebarError}
              />
            </SplitItem>
            <SplitItem isFilled style={{ position: 'relative', height: '100%', minHeight: 0 }}>
              <PreviewPane
                key={selectedDiagram}
                diagramName={selectedDiagram}
                refreshKey={refreshKey}
                hasOutput={diagrams.find(d => d.name === selectedDiagram)?.hasOutput ?? false}
                isStale={previewStale}
                onRender={handleRender}
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
      <StatusBar validation={validation} lastSaved={lastSaved} outputUrl={previewStale ? undefined : outputUrl} isPreviewStale={previewStale} />
      <CreateDiagramModal isOpen={createOpen} onClose={() => setCreateOpen(false)} onCreate={handleCreate} />
    </Page>
  );
}
