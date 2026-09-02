import { describe, expect, it } from 'vitest';
import rendererSource from '../../templates/renderer.js?raw';

const renderData = {
  meta: { title: 'Test diagram', theme: 'light' },
  layout: {
    width: 1000,
    height: 500,
    nodes: [],
    edges: [],
    groups: [],
  },
  flows: [],
};

function renderDiagram() {
  document.body.innerHTML = `
    <main id="diagram-container"></main>
    <aside id="flow-controls" hidden></aside>
    <aside id="magnification-controls" aria-label="Magnification controls"></aside>
    <div id="tooltip" hidden></div>
    <script id="diagram-data" type="application/json">${JSON.stringify(renderData)}</script>
  `;
  window.eval(rendererSource);

  const svg = document.querySelector<SVGSVGElement>('svg')!;
  Object.defineProperty(svg, 'getBoundingClientRect', {
    value: () => ({ left: 0, top: 0, width: 1000, height: 500 }),
  });
  return svg;
}

describe('shared renderer magnification controls', () => {
  it('shows relative wheel magnification, accepts an entry, and resets the centered view', () => {
    const svg = renderDiagram();
    const input = document.getElementById('magnification-input') as HTMLInputElement;
    const reset = document.getElementById('reset-view-button') as HTMLButtonElement;
    const initialViewBox = svg.getAttribute('viewBox');

    expect(input).toHaveValue('0%');

    svg.dispatchEvent(new WheelEvent('wheel', {
      bubbles: true,
      cancelable: true,
      clientX: 500,
      clientY: 250,
      deltaY: -1,
    }));
    expect(input).toHaveValue('11%');

    input.value = '100%';
    input.dispatchEvent(new KeyboardEvent('keydown', { bubbles: true, key: 'Enter' }));
    expect(svg.getAttribute('viewBox')).toBe('250 125 500 250');

    reset.click();
    expect(input).toHaveValue('0%');
    expect(svg.getAttribute('viewBox')).toBe(initialViewBox);
  });
});
