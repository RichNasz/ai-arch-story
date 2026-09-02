(function() {
  'use strict';

  var data = JSON.parse(document.getElementById('diagram-data').textContent);
  var layout = data.layout;
  var flows = data.flows;
  var meta = data.meta;
  var shapeOverrides = data.shapeOverrides || {};
  var isDark = meta.theme === 'dark' ||
    (meta.theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches);

  var container = document.getElementById('diagram-container');
  var tooltip = document.getElementById('tooltip');

  var NS = 'http://www.w3.org/2000/svg';

  // --- SVG Creation Helpers ---
  function el(tag, attrs, parent) {
    var e = document.createElementNS(NS, tag);
    if (attrs) Object.keys(attrs).forEach(function(k) { e.setAttribute(k, attrs[k]); });
    if (parent) parent.appendChild(e);
    return e;
  }

  function text(content, attrs, parent) {
    var t = el('text', attrs, parent);
    t.textContent = content;
    return t;
  }

  // --- Build SVG ---
  var svg = el('svg', {
    'viewBox': '0 0 ' + layout.width + ' ' + layout.height,
    'class': 'arch-diagram',
    'xmlns': NS
  });

  // Defs
  var defs = el('defs', null, svg);

  // Arrow marker — scale with viewport
  var arrowScale = Math.max(1, layout.width / 1920);
  var aw = (10 * arrowScale).toFixed(1), ah = (7 * arrowScale).toFixed(1);
  var marker = el('marker', {
    id: 'arrowhead', markerWidth: aw, markerHeight: ah,
    refX: aw, refY: (3.5 * arrowScale).toFixed(1), orient: 'auto', fill: isDark ? '#64748B' : '#94A3B8'
  }, defs);
  el('polygon', { points: '0 0, ' + aw + ' ' + (3.5*arrowScale).toFixed(1) + ', 0 ' + ah }, marker);

  // Glow filter for flow particles
  var filter = el('filter', { id: 'glow', x: '-50%', y: '-50%', width: '200%', height: '200%' }, defs);
  var blur = el('feGaussianBlur', { stdDeviation: '3', result: 'blur' }, filter);
  var merge = el('feMerge', null, filter);
  el('feMergeNode', { 'in': 'blur' }, merge);
  el('feMergeNode', { 'in': 'SourceGraphic' }, merge);

  // Shadow filter
  var shadow = el('filter', { id: 'shadow', x: '-10%', y: '-10%', width: '120%', height: '130%' }, defs);
  el('feDropShadow', { dx: '0', dy: '2', stdDeviation: '3', 'flood-opacity': '0.08' }, shadow);

  // Layers
  var layerGroups = el('g', { 'class': 'layer-groups' }, svg);
  var layerEdges = el('g', { 'class': 'layer-edges' }, svg);
  var layerNodes = el('g', { 'class': 'layer-nodes' }, svg);
  var layerLabels = el('g', { 'class': 'layer-labels' }, svg);
  var layerFlow = el('g', { 'class': 'layer-flow' }, svg);

  // --- Render Groups ---
  layout.groups.forEach(function(group) {
    var g = el('g', { 'class': 'group', 'data-id': group.id, 'data-depth': group.depth }, layerGroups);
    var opacity = Math.max(0.25, 0.5 - group.depth * 0.1);
    var borderWidth = Math.max(0.8, 1.5 - group.depth * 0.3);
    var fillColor = group.style && group.style.color ? group.style.color :
      (isDark ? 'rgba(30,41,59,' + opacity + ')' : 'rgba(248,250,252,' + opacity + ')');
    var borderColor = isDark ? '#475569' : '#CBD5E1';
    var borderStyle = group.style && group.style.border === 'dashed' ? '6,4' : 'none';

    el('rect', {
      'class': 'group-boundary',
      x: group.x, y: group.y, width: group.width, height: group.height,
      rx: '12', fill: fillColor,
      stroke: borderColor, 'stroke-width': borderWidth,
      'stroke-dasharray': borderStyle
    }, g);

    var groupFontSize = Math.min(Math.max(group.height * 0.06, 14), 28);
    text(group.label, {
      'class': 'group-label',
      x: group.x + groupFontSize * 1.2, y: group.y + groupFontSize * 1.5,
      fill: isDark ? '#94A3B8' : '#64748B',
      'font-size': groupFontSize.toFixed(1), 'font-weight': '600'
    }, g);
  });

  // --- Render Edges ---
  layout.edges.forEach(function(edge) {
    var g = el('g', { 'class': 'edge', 'data-id': edge.id }, layerEdges);
    var pathId = 'edge-path-' + edge.id;

    var edgeStroke = Math.max(1.5, layout.width / 1500);
    el('path', {
      id: pathId, d: edge.path, 'class': 'edge-path',
      fill: 'none', stroke: isDark ? '#64748B' : '#94A3B8', 'stroke-width': edgeStroke.toFixed(1),
      'marker-end': edge.direction !== 'none' ? 'url(#arrowhead)' : '',
      'marker-start': edge.direction === 'two-way' ? 'url(#arrowhead)' : ''
    }, g);

    if (edge.label && edge.label_position) {
      var lg = el('g', { 'class': 'edge-label-group' }, layerLabels);
      var edgeFontSize = Math.max(layout.width * 0.005, 11);
      var charW = edgeFontSize * 0.6;
      var pillW = edge.label.length * charW + edgeFontSize;
      var pillH = edgeFontSize * 1.6;
      el('rect', {
        x: edge.label_position.x - pillW / 2,
        y: edge.label_position.y - pillH / 2,
        width: pillW, height: pillH, rx: (pillH / 2).toFixed(0),
        fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: isDark ? '#334155' : '#E2E8F0', 'stroke-width': '0.5',
        opacity: '0.95'
      }, lg);

      text(edge.label, {
        x: edge.label_position.x, y: edge.label_position.y + edgeFontSize * 0.3,
        'text-anchor': 'middle',
        fill: isDark ? '#94A3B8' : '#64748B',
        'font-size': edgeFontSize.toFixed(1)
      }, lg);
    }
  });

  // --- SVG Shape Override Renderer ---
  function renderSvgShape(g, node, svgContent) {
    var parser = new DOMParser();
    var doc = parser.parseFromString(svgContent, 'image/svg+xml');
    var svgEl = doc.documentElement;
    var vb = svgEl.getAttribute('viewBox');
    if (!vb) vb = '0 0 100 100';
    var foreignSvg = document.createElementNS(NS, 'svg');
    foreignSvg.setAttribute('x', node.x);
    foreignSvg.setAttribute('y', node.y);
    foreignSvg.setAttribute('width', node.width);
    foreignSvg.setAttribute('height', node.height);
    foreignSvg.setAttribute('viewBox', vb);
    foreignSvg.style.color = node.accent_color;
    foreignSvg.style.overflow = 'visible';
    var children = svgEl.childNodes;
    for (var i = 0; i < children.length; i++) {
      if (children[i].nodeType === 1) {
        foreignSvg.appendChild(document.importNode(children[i], true));
      }
    }
    g.appendChild(foreignSvg);
  }

  // --- Node Shape Renderers ---
  var shapes = {
    'rounded-rect': function(g, n) {
      el('rect', {
        x: n.x, y: n.y, width: n.width, height: n.height, rx: '8',
        fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: isDark ? '#334155' : '#E2E8F0', 'stroke-width': '1.5',
        filter: 'url(#shadow)'
      }, g);
      // Accent bar
      el('rect', {
        x: n.x, y: n.y, width: '3', height: n.height, rx: '1.5',
        fill: n.accent_color
      }, g);
    },
    'cylinder': function(g, n) {
      var ry = 8;
      var path = 'M' + n.x + ',' + (n.y + ry) +
        ' A' + (n.width/2) + ',' + ry + ' 0 0,1 ' + (n.x + n.width) + ',' + (n.y + ry) +
        ' V' + (n.y + n.height - ry) +
        ' A' + (n.width/2) + ',' + ry + ' 0 0,1 ' + n.x + ',' + (n.y + n.height - ry) + ' Z';
      el('path', {
        d: path, fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: isDark ? '#334155' : '#E2E8F0', 'stroke-width': '1.5',
        filter: 'url(#shadow)'
      }, g);
      // Top ellipse
      el('ellipse', {
        cx: n.x + n.width/2, cy: n.y + ry, rx: n.width/2, ry: ry,
        fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: isDark ? '#334155' : '#E2E8F0', 'stroke-width': '1.5'
      }, g);
      el('rect', { x: n.x, y: n.y, width: '3', height: n.height, rx: '1.5', fill: n.accent_color }, g);
    },
    'diamond': function(g, n) {
      var cx = n.x + n.width/2, cy = n.y + n.height/2;
      var pts = cx+','+ n.y +' '+ (n.x+n.width)+','+cy +' '+ cx+','+(n.y+n.height) +' '+ n.x+','+cy;
      el('polygon', {
        points: pts, fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: isDark ? '#334155' : '#E2E8F0', 'stroke-width': '1.5',
        filter: 'url(#shadow)'
      }, g);
    },
    'hexagon': function(g, n) {
      var cx = n.x + n.width/2, inset = n.width * 0.25;
      var pts = (n.x+inset)+','+n.y +' '+ (n.x+n.width-inset)+','+n.y +' '+
        (n.x+n.width)+','+(n.y+n.height/2) +' '+ (n.x+n.width-inset)+','+(n.y+n.height) +' '+
        (n.x+inset)+','+(n.y+n.height) +' '+ n.x+','+(n.y+n.height/2);
      el('polygon', {
        points: pts, fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: isDark ? '#334155' : '#E2E8F0', 'stroke-width': '1.5',
        filter: 'url(#shadow)'
      }, g);
    },
    'parallelogram': function(g, n) {
      var skew = 15;
      var pts = (n.x+skew)+','+n.y +' '+ (n.x+n.width)+','+n.y +' '+
        (n.x+n.width-skew)+','+(n.y+n.height) +' '+ n.x+','+(n.y+n.height);
      el('polygon', {
        points: pts, fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: isDark ? '#334155' : '#E2E8F0', 'stroke-width': '1.5',
        filter: 'url(#shadow)'
      }, g);
      el('rect', { x: n.x+skew, y: n.y, width: '3', height: n.height, rx: '1.5', fill: n.accent_color }, g);
    },
    'dashed-rect': function(g, n) {
      el('rect', {
        x: n.x, y: n.y, width: n.width, height: n.height, rx: '8',
        fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: isDark ? '#475569' : '#94A3B8', 'stroke-width': '1.5',
        'stroke-dasharray': '6,4', filter: 'url(#shadow)'
      }, g);
    },
    'browser': function(g, n) {
      el('rect', {
        x: n.x, y: n.y, width: n.width, height: n.height, rx: '8',
        fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: isDark ? '#334155' : '#E2E8F0', 'stroke-width': '1.5',
        filter: 'url(#shadow)'
      }, g);
      // Browser chrome bar
      el('rect', {
        x: n.x, y: n.y, width: n.width, height: '12', rx: '8',
        fill: n.accent_color, opacity: '0.15'
      }, g);
      var dotY = n.y + 6;
      [8, 16, 24].forEach(function(dx) {
        el('circle', { cx: n.x + dx, cy: dotY, r: '2', fill: n.accent_color, opacity: '0.4' }, g);
      });
    },
    'person': function(g, n) {
      var cx = n.x + n.width/2, top = n.y + 8;
      el('circle', { cx: cx, cy: top + 10, r: '10', fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: n.accent_color, 'stroke-width': '1.5' }, g);
      el('path', {
        d: 'M'+(cx-18)+','+(top+45)+' Q'+(cx-18)+','+(top+22)+' '+cx+','+(top+22)+
          ' Q'+(cx+18)+','+(top+22)+' '+(cx+18)+','+(top+45),
        fill: 'none', stroke: n.accent_color, 'stroke-width': '1.5'
      }, g);
    },
    'folder': function(g, n) {
      shapes['rounded-rect'](g, n);
    },
    'rect': function(g, n) {
      el('rect', {
        x: n.x, y: n.y, width: n.width, height: n.height,
        fill: isDark ? '#1E293B' : '#FFFFFF',
        stroke: isDark ? '#334155' : '#E2E8F0', 'stroke-width': '1.5',
        filter: 'url(#shadow)'
      }, g);
    }
  };

  // --- Render Nodes ---
  layout.nodes.forEach(function(node) {
    var g = el('g', {
      'class': 'node', 'data-id': node.id, 'data-type': node.type
    }, layerNodes);

    if (shapeOverrides[node.shape]) {
      renderSvgShape(g, node, shapeOverrides[node.shape]);
    } else {
      var shapeFn = shapes[node.shape] || shapes['rounded-rect'];
      shapeFn(g, node);
    }

    // Label — scale font to fit node
    var fontSize = Math.min(node.height * 0.35, node.width / (node.label.length * 0.65));
    fontSize = Math.max(fontSize, 10);
    var labelY = node.shape === 'person' ? node.y + node.height - 5 : node.y + node.height / 2 + fontSize * 0.35;
    text(node.label, {
      x: node.x + node.width / 2, y: labelY,
      'text-anchor': 'middle',
      fill: isDark ? '#F1F5F9' : '#1E293B',
      'font-size': fontSize.toFixed(1), 'font-weight': '500'
    }, g);

    // Tooltip interaction
    g.addEventListener('mouseenter', function(e) {
      var desc = node.metadata && node.metadata.description ? node.metadata.description : '';
      var tech = node.metadata && node.metadata.technology ? node.metadata.technology : '';
      if (!desc && !tech) return;
      tooltip.innerHTML = '<strong>' + node.label + '</strong>' +
        (tech ? '<br><span class="tt-tech">' + tech + '</span>' : '') +
        (desc ? '<br>' + desc : '');
      tooltip.hidden = false;
    });
    g.addEventListener('mousemove', function(e) {
      tooltip.style.left = (e.clientX + 12) + 'px';
      tooltip.style.top = (e.clientY + 12) + 'px';
    });
    g.addEventListener('mouseleave', function() {
      tooltip.hidden = true;
    });
  });

  // --- Flow Stepper Engine ---
  if (flows && flows.length > 0) {
    var controls = document.getElementById('flow-controls');
    controls.hidden = false;
    controls.setAttribute('data-fc-mode', 'floating');

    var panelMode = 'floating';
    var previousMode = 'floating';
    var restoreBtn = null;
    var btnDock = null;

    var activeFlowIndex = 0;
    var currentStep = -1;
    var isPlaying = false;
    var autoplayTimer = null;
    var stepDelay = 2000;
    var particleEl = null;
    var edgeOverlays = [];

    // Build node lookup for from/to labels
    var nodeById = {};
    layout.nodes.forEach(function(n) { nodeById[n.id] = n; });

    // --- Edge highlight helpers ---
    function clearHighlights() {
      edgeOverlays.forEach(function(ov) { ov.parentNode.removeChild(ov); });
      edgeOverlays = [];
      if (particleEl) { particleEl.setAttribute('opacity', '0'); }
    }

    function highlightEdge(flow, stepIdx) {
      var step = flow.steps[stepIdx];
      if (!step) return;
      var pathEl = document.getElementById('edge-path-' + step.edge);
      if (!pathEl) return;

      var overlay = el('path', {
        d: pathEl.getAttribute('d'), fill: 'none',
        stroke: flow.color, 'stroke-width': '3.5', opacity: '0.8',
        'class': 'edge-path-highlight'
      }, layerFlow);
      edgeOverlays.push(overlay);

      // Animate particle along the edge
      if (!particleEl) {
        particleEl = el('circle', {
          r: '5', fill: flow.color, filter: 'url(#glow)', opacity: '0'
        }, layerFlow);
      }
      particleEl.setAttribute('fill', flow.color);
      particleEl.setAttribute('opacity', '1');

      var pathLen = pathEl.getTotalLength();
      var startTime = null;
      var dur = 800;
      function frame(ts) {
        if (!startTime) startTime = ts;
        var progress = Math.min((ts - startTime) / dur, 1);
        var ease = progress < 0.5 ? 2*progress*progress : 1 - Math.pow(-2*progress+2, 2)/2;
        var pt = pathEl.getPointAtLength(ease * pathLen);
        particleEl.setAttribute('cx', pt.x);
        particleEl.setAttribute('cy', pt.y);
        if (progress < 1) requestAnimationFrame(frame);
        else particleEl.setAttribute('opacity', '0');
      }
      requestAnimationFrame(frame);
    }

    // --- Draggable panel ---
    var isDragging = false, dragOffX = 0, dragOffY = 0;

    function onDragStart(e) {
      if (panelMode !== 'floating') return;
      if (e.target.closest('button, input, select, label, .fc-step')) return;
      isDragging = true;
      var rect = controls.getBoundingClientRect();
      dragOffX = e.clientX - rect.left;
      dragOffY = e.clientY - rect.top;
      controls.style.transition = 'none';
      e.preventDefault();
    }

    function onDragMove(e) {
      if (!isDragging) return;
      var x = e.clientX - dragOffX;
      var y = e.clientY - dragOffY;
      x = Math.max(0, Math.min(x, window.innerWidth - controls.offsetWidth));
      y = Math.max(0, Math.min(y, window.innerHeight - controls.offsetHeight));
      controls.style.left = x + 'px';
      controls.style.top = y + 'px';
      controls.style.right = 'auto';
      controls.style.bottom = 'auto';
    }

    function onDragEnd() {
      isDragging = false;
      controls.style.transition = '';
    }

    controls.addEventListener('mousedown', onDragStart);
    window.addEventListener('mousemove', onDragMove);
    window.addEventListener('mouseup', onDragEnd);

    controls.addEventListener('touchstart', function(e) {
      if (panelMode !== 'floating') return;
      if (e.target.closest('button, input, select, label, .fc-step')) return;
      isDragging = true;
      var rect = controls.getBoundingClientRect();
      dragOffX = e.touches[0].clientX - rect.left;
      dragOffY = e.touches[0].clientY - rect.top;
      controls.style.transition = 'none';
    }, { passive: true });

    controls.addEventListener('touchmove', function(e) {
      if (!isDragging) return;
      var x = e.touches[0].clientX - dragOffX;
      var y = e.touches[0].clientY - dragOffY;
      x = Math.max(0, Math.min(x, window.innerWidth - controls.offsetWidth));
      y = Math.max(0, Math.min(y, window.innerHeight - controls.offsetHeight));
      controls.style.left = x + 'px';
      controls.style.top = y + 'px';
      controls.style.right = 'auto';
      controls.style.bottom = 'auto';
    }, { passive: true });

    controls.addEventListener('touchend', onDragEnd, { passive: true });

    // --- Mode management ---
    function setMode(newMode) {
      if (newMode === 'minimized') {
        previousMode = panelMode;
      }
      panelMode = newMode;
      controls.setAttribute('data-fc-mode', newMode);

      if (newMode !== 'floating') {
        controls.style.left = '';
        controls.style.top = '';
        controls.style.right = '';
        controls.style.bottom = '';
      } else {
        controls.style.right = '20px';
        controls.style.bottom = '20px';
        controls.style.left = 'auto';
        controls.style.top = 'auto';
      }

      if (newMode === 'minimized') {
        restoreBtn.classList.add('visible');
      } else {
        restoreBtn.classList.remove('visible');
        controls.classList.add('fc-restoring');
        controls.addEventListener('animationend', function handler() {
          controls.classList.remove('fc-restoring');
          controls.removeEventListener('animationend', handler);
        });
      }

      updateModeButtons();
    }

    function updateModeButtons() {
      if (!btnDock) return;
      if (panelMode === 'docked') {
        btnDock.innerHTML = '&#x25F1;';
        btnDock.title = 'Float panel';
      } else {
        btnDock.innerHTML = '&#x25F2;';
        btnDock.title = 'Dock to right';
      }
    }

    // --- Restore button ---
    restoreBtn = document.createElement('button');
    restoreBtn.id = 'fc-restore-btn';
    restoreBtn.title = 'Show flow controls';
    restoreBtn.innerHTML = '&#x25B6;';
    document.body.appendChild(restoreBtn);
    restoreBtn.addEventListener('click', function() {
      setMode(previousMode);
    });

    // --- Build UI ---
    var header = document.createElement('div');
    header.className = 'fc-header';

    var headerLabel = document.createElement('span');
    headerLabel.textContent = 'Flows';
    header.appendChild(headerLabel);

    var modeBtns = document.createElement('span');
    modeBtns.className = 'fc-mode-btns';

    btnDock = document.createElement('button');
    btnDock.className = 'fc-mode-btn';
    btnDock.title = 'Dock to right';
    btnDock.innerHTML = '&#x25F2;';
    btnDock.addEventListener('click', function() {
      setMode(panelMode === 'docked' ? 'floating' : 'docked');
    });
    modeBtns.appendChild(btnDock);

    var btnMinimize = document.createElement('button');
    btnMinimize.className = 'fc-mode-btn';
    btnMinimize.title = 'Minimize';
    btnMinimize.innerHTML = '&#x2500;';
    btnMinimize.addEventListener('click', function() {
      setMode('minimized');
    });
    modeBtns.appendChild(btnMinimize);

    header.appendChild(modeBtns);
    controls.appendChild(header);

    // Flow tabs
    var tabsRow = document.createElement('div');
    tabsRow.className = 'fc-tabs';
    controls.appendChild(tabsRow);

    var tabs = [];
    flows.forEach(function(flow, i) {
      var tab = document.createElement('button');
      tab.className = 'fc-tab' + (i === 0 ? ' active' : '');
      var swatch = document.createElement('span');
      swatch.className = 'fc-tab-swatch';
      swatch.style.background = flow.color;
      tab.appendChild(swatch);
      tab.appendChild(document.createTextNode(flow.label));
      tab.addEventListener('click', function() { selectFlow(i); });
      tabsRow.appendChild(tab);
      tabs.push(tab);
    });

    // Body
    var body = document.createElement('div');
    body.className = 'fc-body';
    controls.appendChild(body);

    var descEl = document.createElement('div');
    descEl.className = 'fc-desc';
    body.appendChild(descEl);

    var stepper = document.createElement('div');
    stepper.className = 'fc-stepper';
    body.appendChild(stepper);

    // Transport bar
    var transport = document.createElement('div');
    transport.className = 'fc-transport';
    controls.appendChild(transport);

    var transportBtns = document.createElement('div');
    transportBtns.className = 'fc-transport-btns';
    transport.appendChild(transportBtns);

    var btnReset = document.createElement('button');
    btnReset.className = 'fc-btn'; btnReset.title = 'Reset';
    btnReset.innerHTML = '&#x23EE;'; // ⏮
    transportBtns.appendChild(btnReset);

    var btnPrev = document.createElement('button');
    btnPrev.className = 'fc-btn'; btnPrev.title = 'Previous step';
    btnPrev.innerHTML = '&#x25C0;'; // ◀
    transportBtns.appendChild(btnPrev);

    var btnPlay = document.createElement('button');
    btnPlay.className = 'fc-btn'; btnPlay.title = 'Autoplay';
    btnPlay.innerHTML = '&#x25B6;'; // ▶
    transportBtns.appendChild(btnPlay);

    var btnNext = document.createElement('button');
    btnNext.className = 'fc-btn'; btnNext.title = 'Next step';
    btnNext.innerHTML = '&#x25B6;'; // ▶
    transportBtns.appendChild(btnNext);

    var progress = document.createElement('span');
    progress.className = 'fc-progress';
    transportBtns.appendChild(progress);

    var delayWrap = document.createElement('div');
    delayWrap.className = 'fc-delay';
    var delayLabel = document.createElement('label');
    delayLabel.textContent = 'Delay';
    delayWrap.appendChild(delayLabel);
    var delaySelect = document.createElement('select');
    [{ v: 1000, t: '1s' }, { v: 2000, t: '2s' }, { v: 3000, t: '3s' }, { v: 5000, t: '5s' }].forEach(function(opt) {
      var o = document.createElement('option');
      o.value = opt.v; o.textContent = opt.t;
      if (opt.v === stepDelay) o.selected = true;
      delaySelect.appendChild(o);
    });
    delaySelect.addEventListener('change', function() {
      stepDelay = parseInt(delaySelect.value, 10);
      if (isPlaying) { stopAutoplay(); startAutoplay(); }
    });
    delayWrap.appendChild(delaySelect);
    transport.appendChild(delayWrap);

    // --- Step rendering ---
    var stepEls = [];

    function renderSteps(flow) {
      stepper.innerHTML = '';
      stepEls = [];
      descEl.textContent = flow.description || '';
      descEl.hidden = !flow.description;

      flow.steps.forEach(function(step, i) {
        var row = document.createElement('div');
        row.className = 'fc-step';
        row.addEventListener('click', function() { goToStep(i); });

        var num = document.createElement('span');
        num.className = 'fc-step-num';
        num.textContent = (i + 1);
        row.appendChild(num);

        var content = document.createElement('div');
        content.className = 'fc-step-content';

        var lbl = document.createElement('div');
        lbl.className = 'fc-step-label';
        var fromNode = nodeById[step.from_node];
        var toNode = nodeById[step.to_node];
        var labelText = step.label || ((fromNode ? fromNode.label : step.from_node) + ' → ' + (toNode ? toNode.label : step.to_node));
        lbl.textContent = labelText;
        content.appendChild(lbl);

        if (step.description) {
          var desc = document.createElement('div');
          desc.className = 'fc-step-desc';
          desc.textContent = step.description;
          content.appendChild(desc);
        }

        row.appendChild(content);
        stepper.appendChild(row);
        stepEls.push(row);
      });
    }

    function updateStepUI() {
      var flow = flows[activeFlowIndex];
      stepEls.forEach(function(el, i) {
        el.className = 'fc-step' +
          (i === currentStep ? ' active' : '') +
          (i < currentStep ? ' done' : '');
      });
      progress.textContent = (currentStep + 1) + ' / ' + flow.steps.length;
      btnPrev.disabled = currentStep <= 0;
      btnNext.disabled = currentStep >= flow.steps.length - 1;
      btnReset.disabled = currentStep < 0;
    }

    // --- Navigation ---
    function goToStep(idx) {
      var flow = flows[activeFlowIndex];
      if (idx < 0 || idx >= flow.steps.length) return;
      currentStep = idx;
      clearHighlights();
      // Highlight all edges up to and including current step
      for (var j = 0; j <= idx; j++) {
        highlightEdge(flow, j);
      }
      updateStepUI();
      // Scroll active step into view
      if (stepEls[idx]) stepEls[idx].scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }

    function nextStep() {
      var flow = flows[activeFlowIndex];
      if (currentStep < flow.steps.length - 1) {
        goToStep(currentStep + 1);
      } else {
        stopAutoplay();
      }
    }

    function prevStep() {
      if (currentStep > 0) goToStep(currentStep - 1);
    }

    function resetSteps() {
      stopAutoplay();
      currentStep = -1;
      clearHighlights();
      updateStepUI();
    }

    // --- Autoplay ---
    function startAutoplay() {
      isPlaying = true;
      btnPlay.classList.add('playing');
      btnPlay.innerHTML = '&#x23F8;'; // ⏸
      btnPlay.title = 'Pause';
      tick();
    }

    function stopAutoplay() {
      isPlaying = false;
      btnPlay.classList.remove('playing');
      btnPlay.innerHTML = '&#x25B6;'; // ▶
      btnPlay.title = 'Autoplay';
      if (autoplayTimer) { clearTimeout(autoplayTimer); autoplayTimer = null; }
    }

    function tick() {
      if (!isPlaying) return;
      nextStep();
      var flow = flows[activeFlowIndex];
      if (currentStep < flow.steps.length - 1) {
        autoplayTimer = setTimeout(tick, stepDelay);
      } else {
        stopAutoplay();
      }
    }

    // --- Flow selection ---
    function selectFlow(idx) {
      activeFlowIndex = idx;
      tabs.forEach(function(t, i) {
        t.className = 'fc-tab' + (i === idx ? ' active' : '');
      });
      stopAutoplay();
      currentStep = -1;
      clearHighlights();
      renderSteps(flows[idx]);
      updateStepUI();
    }

    // --- Wire up transport buttons ---
    btnReset.addEventListener('click', resetSteps);
    btnPrev.addEventListener('click', prevStep);
    btnNext.addEventListener('click', nextStep);
    btnPlay.addEventListener('click', function() {
      if (isPlaying) {
        stopAutoplay();
      } else {
        var flow = flows[activeFlowIndex];
        if (currentStep >= flow.steps.length - 1) currentStep = -1;
        startAutoplay();
      }
    });

    // Initialize
    selectFlow(0);
  }

  // --- Pan & Zoom ---
  var initialViewBox = { x: 0, y: 0, w: layout.width, h: layout.height };
  var viewBox = { x: initialViewBox.x, y: initialViewBox.y, w: initialViewBox.w, h: initialViewBox.h };
  var isPanning = false, panStart = { x: 0, y: 0 }, panViewStart = { x: 0, y: 0 };
  var MIN_MAGNIFICATION = -90;
  var MAX_MAGNIFICATION = 400;
  var magnificationInput;

  function updateViewBox() {
    svg.setAttribute('viewBox', viewBox.x + ' ' + viewBox.y + ' ' + viewBox.w + ' ' + viewBox.h);
    updateMagnificationDisplay();
  }

  function currentMagnification() {
    return ((initialViewBox.w / viewBox.w) - 1) * 100;
  }

  function updateMagnificationDisplay() {
    if (magnificationInput) {
      magnificationInput.value = Math.round(currentMagnification()) + '%';
    }
  }

  function setMagnification(magnification) {
    var clamped = Math.max(MIN_MAGNIFICATION, Math.min(MAX_MAGNIFICATION, magnification));
    var scale = 1 + clamped / 100;
    var centerX = viewBox.x + viewBox.w / 2;
    var centerY = viewBox.y + viewBox.h / 2;
    viewBox.w = initialViewBox.w / scale;
    viewBox.h = initialViewBox.h / scale;
    viewBox.x = centerX - viewBox.w / 2;
    viewBox.y = centerY - viewBox.h / 2;
    updateViewBox();
  }

  function resetView() {
    viewBox.x = initialViewBox.x;
    viewBox.y = initialViewBox.y;
    viewBox.w = initialViewBox.w;
    viewBox.h = initialViewBox.h;
    updateViewBox();
  }

  function buildMagnificationControls() {
    var controls = document.getElementById('magnification-controls');
    if (!controls) return;

    var label = document.createElement('label');
    label.htmlFor = 'magnification-input';
    label.textContent = 'Magnification';
    controls.appendChild(label);

    magnificationInput = document.createElement('input');
    magnificationInput.id = 'magnification-input';
    magnificationInput.type = 'text';
    magnificationInput.inputMode = 'numeric';
    magnificationInput.setAttribute('aria-label', 'Magnification');
    controls.appendChild(magnificationInput);

    function applyInput() {
      var match = magnificationInput.value.trim().match(/^(-?\d+)%?$/);
      if (!match) {
        updateMagnificationDisplay();
        return;
      }
      setMagnification(parseInt(match[1], 10));
    }

    magnificationInput.addEventListener('keydown', function(e) {
      if (e.key === 'Enter') {
        e.preventDefault();
        applyInput();
        magnificationInput.blur();
      }
    });
    magnificationInput.addEventListener('blur', applyInput);

    var resetButton = document.createElement('button');
    resetButton.id = 'reset-view-button';
    resetButton.type = 'button';
    resetButton.textContent = 'Reset view';
    resetButton.addEventListener('click', resetView);
    controls.appendChild(resetButton);
    updateMagnificationDisplay();
  }

  svg.addEventListener('mousedown', function(e) {
    if (e.target.closest('.node')) return;
    isPanning = true;
    panStart = { x: e.clientX, y: e.clientY };
    panViewStart = { x: viewBox.x, y: viewBox.y };
    svg.style.cursor = 'grabbing';
  });

  window.addEventListener('mousemove', function(e) {
    if (!isPanning) return;
    var rect = svg.getBoundingClientRect();
    var scale = viewBox.w / rect.width;
    viewBox.x = panViewStart.x - (e.clientX - panStart.x) * scale;
    viewBox.y = panViewStart.y - (e.clientY - panStart.y) * scale;
    updateViewBox();
  });

  window.addEventListener('mouseup', function() {
    isPanning = false;
    svg.style.cursor = 'grab';
  });

  svg.addEventListener('wheel', function(e) {
    e.preventDefault();
    var rect = svg.getBoundingClientRect();
    var mx = (e.clientX - rect.left) / rect.width;
    var my = (e.clientY - rect.top) / rect.height;
    var factor = e.deltaY > 0 ? 1.1 : 0.9;
    var targetMagnification = currentMagnification();
    targetMagnification = ((1 + targetMagnification / 100) / factor - 1) * 100;
    targetMagnification = Math.max(MIN_MAGNIFICATION, Math.min(MAX_MAGNIFICATION, targetMagnification));
    var targetScale = 1 + targetMagnification / 100;
    var newW = initialViewBox.w / targetScale;
    var newH = initialViewBox.h / targetScale;
    viewBox.x += (viewBox.w - newW) * mx;
    viewBox.y += (viewBox.h - newH) * my;
    viewBox.w = newW;
    viewBox.h = newH;
    updateViewBox();
  }, { passive: false });

  // Touch support
  svg.addEventListener('touchstart', function(e) {
    if (e.touches.length === 1) {
      isPanning = true;
      panStart = { x: e.touches[0].clientX, y: e.touches[0].clientY };
      panViewStart = { x: viewBox.x, y: viewBox.y };
    }
  }, { passive: true });

  svg.addEventListener('touchmove', function(e) {
    if (!isPanning || e.touches.length !== 1) return;
    var rect = svg.getBoundingClientRect();
    var scale = viewBox.w / rect.width;
    viewBox.x = panViewStart.x - (e.touches[0].clientX - panStart.x) * scale;
    viewBox.y = panViewStart.y - (e.touches[0].clientY - panStart.y) * scale;
    updateViewBox();
  }, { passive: true });

  svg.addEventListener('touchend', function() { isPanning = false; }, { passive: true });

  // --- Mount SVG ---
  container.appendChild(svg);
  svg.style.cursor = 'grab';
  buildMagnificationControls();
})();
