/* smart-daw daw.js — dock system, pane float/dock, drag-drop, tooltip, context menu */
/* Tokens strictly from smart.toml; skeuomorphic feel only in knobs/faders. */

(function(){
  const $ = s => document.querySelector(s);
  const $$ = s => Array.from(document.querySelectorAll(s));

  /* ============ DOCK SYSTEM ============ */
  const dockSlots = {
    browser:  { x: 0,  y: 0,  w: 280, h: '100%' },
    channel:  { x: 280, y: 0,  w: 'calc(50% - 140px)', h: 'calc(50% - 6px)' },
    piano:    { x: '50%', y: 0,  w: '50%', h: '100%' },
    transport:{ x: 280, y: '50%', w: 'calc(50% - 140px)', h: 64 },
    mixer:    { x: 280, y: 'calc(50% + 64px)', w: 'calc(50% - 140px)', h: 'calc(50% - 70px)' },
  };

  /* ============ PANE MANAGEMENT ============ */
  function floatPane(pane){
    pane.classList.add('floating');
    pane.classList.remove('docked');
    pane.style.position = 'fixed';
    pane.style.left = '220px';
    pane.style.top = '140px';
    pane.style.width = '420px';
    pane.style.height = '460px';
    pane.style.zIndex = '60';
  }

  function dockPane(pane, slot){
    pane.classList.remove('floating');
    pane.classList.add('docked');
    pane.style.position = '';
    pane.style.left = '';
    pane.style.top = '';
    pane.style.width = '';
    pane.style.height = '';
    pane.style.zIndex = '';
    if (slot && dockSlots[slot]) {
      const s = dockSlots[slot];
      pane.style.gridArea = slot;
    }
  }

  function toggleFloatDock(pane){
    if (pane.classList.contains('floating')) {
      dockPane(pane);
    } else {
      floatPane(pane);
    }
  }

  // Make pane headers draggable when floating
  $$('.daw-pane-header').forEach(header => {
    const pane = header.closest('.daw-pane');
    const floatBtn = header.querySelector('[title="Float/dock"]');
    if (floatBtn) {
      floatBtn.addEventListener('click', () => toggleFloatDock(pane));
    }
    let dragging = false, startX, startY, startLeft, startTop;
    header.addEventListener('mousedown', e => {
      if (e.target.tagName === 'BUTTON' || e.target.tagName === 'SELECT' || e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;
      if (!pane.classList.contains('floating')) return;
      dragging = true;
      startX = e.clientX; startY = e.clientY;
      const cs = getComputedStyle(pane);
      startLeft = parseFloat(cs.left) || 0; startTop = parseFloat(cs.top) || 0;
      pane.style.zIndex = 70;
      e.preventDefault();
    });
    window.addEventListener('mousemove', e => {
      if (!dragging) return;
      pane.style.left = (startLeft + e.clientX - startX) + 'px';
      pane.style.top = (startTop + e.clientY - startY) + 'px';
    });
    window.addEventListener('mouseup', () => {
      if (dragging) {
        dragging = false;
        pane.style.zIndex = 60;
      }
    });
  });

  /* ============ CLOSE BUTTONS ============ */
  $$('[title="Close"]').forEach(btn => {
    btn.addEventListener('click', () => {
      const pane = btn.closest('.daw-pane');
      if (pane) pane.style.display = 'none';
    });
  });

  /* ============ KEYBOARD SHORTCUTS — sidebar ============ */
  document.addEventListener('keydown', e => {
    if ((e.metaKey || e.ctrlKey) && e.key === '\\') {
      e.preventDefault();
      const windowEl = $('.window');
      if (windowEl) windowEl.classList.toggle('is-collapsed');
    }
  });

  /* ============ RESIZE HANDLES ============ */
  function addResizeHandles(){
    const panes = $$('.daw-pane.docked');
    panes.forEach(pane => {
      const existing = pane.querySelector('.resize-handle');
      if (existing) existing.remove();
      const handle = document.createElement('div');
      handle.className = 'resize-handle h';
      handle.style.cssText = 'position:absolute;top:0;bottom:0;width:6px;cursor:ew-resize;z-index:5;left:100%;transform:translateX(-50%);';
      pane.style.position = 'relative';
      pane.appendChild(handle);
    });
  }

  /* ============ TOOLTIP SYSTEM ============ */
  const tooltip = $('#daw-tooltip');
  let ttTimer;
  function showTooltip(el, e){
    clearTimeout(ttTimer);
    ttTimer = setTimeout(() => {
      tooltip.textContent = (el.dataset.knob || el.dataset.fader || el.title || '');
      tooltip.classList.add('show');
      positionTooltip(e);
    }, 400);
  }
  function hideTooltip(){
    clearTimeout(ttTimer);
    tooltip.classList.remove('show');
  }
  function positionTooltip(e){
    tooltip.style.left = (e.clientX + 14) + 'px';
    tooltip.style.top = (e.clientY + 14) + 'px';
  }

  $$('[title]').forEach(el => {
    el.addEventListener('mouseenter', e => showTooltip(el, e));
    el.addEventListener('mouseleave', hideTooltip);
    el.addEventListener('mousemove', e => {
      if (tooltip.classList.contains('show')) positionTooltip(e);
    });
  });

  /* ============ DRAG AND DROP ============ */
  function setupDragDrop(){
    const draggables = $$('.browser-item, .rack-step, .piano-note, .playlist-clip');
    draggables.forEach(item => {
      item.setAttribute('draggable', 'true');
      item.addEventListener('dragstart', e => {
        e.dataTransfer.setData('text/plain', item.dataset.id || item.textContent || '');
        e.dataTransfer.effectAllowed = 'copyMove';
        item.style.opacity = '0.5';
      });
      item.addEventListener('dragend', e => {
        item.style.opacity = '';
      });
    });

    const dropZones = $$('.daw-pane-content, .piano-grid, .playlist-area');
    dropZones.forEach(zone => {
      zone.addEventListener('dragover', e => {
        e.preventDefault();
        zone.style.outline = '2px dashed var(--accent)';
        zone.style.outlineOffset = '-2px';
      });
      zone.addEventListener('dragleave', e => {
        zone.style.outline = '';
        zone.style.outlineOffset = '';
      });
      zone.addEventListener('drop', e => {
        e.preventDefault();
        zone.style.outline = '';
        zone.style.outlineOffset = '';
        const data = e.dataTransfer.getData('text/plain');
        const zoneId = zone.id || zone.closest('.daw-pane-content')?.id || 'unknown';
        if (window.smartDaw && typeof window.smartDaw.onDrop === 'function') {
          window.smartDaw.onDrop(data, zoneId, e);
        } else {
          console.log('Dropped:', data, 'on', zoneId);
        }
      });
    });
  }

  /* ============ CONTEXT MENU ============ */
  function showContextMenu(x, y, items){
    const existing = $('.context-menu');
    if (existing) existing.remove();

    const menu = document.createElement('div');
    menu.className = 'context-menu menu';
    menu.style.cssText = 'position:fixed;left:' + x + 'px;top:' + y + 'px;z-index:86;min-width:220px;padding:7px';

    items.forEach(item => {
      if (item.separator) {
        const sep = document.createElement('div');
        sep.className = 'msep';
        menu.appendChild(sep);
      } else {
        const mi = document.createElement('button');
        mi.className = 'mi';
        mi.style.cssText = 'display:flex;align-items:center;gap:9px;width:100%;height:33px;padding:0 8px;border-radius:9px;color:var(--ink-2);font-size:12.5px;font-weight:495;letter-spacing:-.003em;text-align:left;white-space:nowrap;transition:background var(--dur-1) ease;background:none;border:none;cursor:default;text-align:left';
        mi.innerHTML = '<span class="ic" style="--ic:16px;color:var(--ink-3)">' + (item.icon || '') + '</span><span class="ml" style="flex:1;min-width:0;overflow:hidden;text-overflow:ellipsis">' + item.label + '</span>' + (item.shortcut ? '<span class="mk" style="flex:none;margin-left:14px;font-size:10px;font-weight:520;letter-spacing:.035em;color:var(--ink-3);font-variant-numeric:tabular-nums">' + item.shortcut + '</span>' : '');
        mi.addEventListener('click', () => { item.action(); menu.remove(); });
        mi.addEventListener('mouseenter', () => { mi.style.background = 'var(--accent-tint)'; });
        mi.addEventListener('mouseleave', () => { mi.style.background = ''; });
        menu.appendChild(mi);
      }
    });

    document.body.appendChild(menu);

    const closeHandler = (e) => {
      if (!menu.contains(e.target)) {
        menu.remove();
        document.removeEventListener('click', closeHandler);
      }
    };
    setTimeout(() => document.addEventListener('click', closeHandler), 0);
  }

  document.addEventListener('contextmenu', e => {
    const note = e.target.closest('.piano-note');
    if (note) {
      e.preventDefault();
      showContextMenu(e.clientX, e.clientY, [
        { label: 'Edit velocity', action: () => { const v = prompt('Velocity (0-127)', 90); if (v) console.log('Set velocity:', v); } },
        { label: 'Duplicate', shortcut: 'Alt+Drag', action: () => console.log('Duplicate note') },
        { separator: true },
        { label: 'Delete', shortcut: 'Del', action: () => console.log('Delete note') },
      ]);
    }
  });

  /* ============ INITIALIZATION ============ */
  function init(){
    addResizeHandles();
    setupDragDrop();
    console.log('Smart DAW dock initialized');
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }

  window.smartDawDock = { floatPane, dockPane, toggleFloatDock, showContextMenu, addResizeHandles, setupDragDrop };
})();
