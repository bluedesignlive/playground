(() => {
'use strict';

/* ============================================================
   Smart Files — prototype behaviour
   Data · render · menus · preview · AI · shortcuts
   ============================================================ */

const TAG = {
  design:   { label: 'Design',   color: '#2f6feb' },
  amapiano: { label: 'Amapiano', color: '#c9861a' },
  archive:  { label: 'Archive',  color: '#2b9b6d' }
};

const NAV = {
  home:      { title: 'Home',        icon: 'home' },
  recents:   { title: 'Recents',     icon: 'clock-circle' },
  desktop:   { title: 'Desktop',     icon: 'monitor' },
  documents: { title: 'Documents',   icon: 'document-text' },
  downloads: { title: 'Downloads',   icon: 'download-minimalistic' },
  pictures:  { title: 'Pictures',    icon: 'gallery' },
  music:     { title: 'Music',       icon: 'music-note' },
  movies:    { title: 'Movies',      icon: 'clapperboard' },
  drive:     { title: 'Smart Drive', icon: 'ssd-square' },
  network:   { title: 'Network',     icon: 'global' },
  cloud:     { title: 'Smart Cloud', icon: 'cloud' }
};

const MB = 1000000;
const KB = 1000;

const FOLDERS = [
  {
    key: 'fold-amapiano', id: 'amapiano', type: 'folder',
    name: 'Amapiano Pack Vol. 1', tag: 'amapiano', ts: 88, kindLabel: 'Folder', date: 'Today, 08:02',
    items: [
      { key: 'file-logdrum-126', name: '01 Log Drum 126 BPM.wav', kind: 'Audio', kindLabel: 'WAV audio', ext: 'WAV', icon: 'music-note', size: '18.4 MB', bytes: 18.4 * MB, date: 'Today, 08:02', ts: 99, tag: 'amapiano' },
      { key: 'file-shaker-128', name: '02 Shaker Groove.wav', kind: 'Audio', kindLabel: 'WAV audio', ext: 'WAV', icon: 'music-note', size: '6.2 MB', bytes: 6.2 * MB, date: 'Today, 08:01', ts: 98, tag: 'amapiano' },
      { key: 'file-pack-flp', name: 'Amapiano Pack Vol. 1.flp', kind: 'Project', kindLabel: 'FL Studio project', ext: 'FLP', icon: 'file', size: '4.8 MB', bytes: 4.8 * MB, date: 'Yesterday, 23:18', ts: 93, tag: 'amapiano' },
      { key: 'file-pack-cover', name: 'Pack Cover.jpg', kind: 'Image', kindLabel: 'JPEG image', ext: 'JPG', icon: 'gallery', thumb: 'poster-01', dims: '1600 × 1600', size: '2.1 MB', bytes: 2.1 * MB, date: 'Oct 30, 18:20', ts: 84, tag: 'amapiano' }
    ]
  },
  {
    key: 'fold-logdrum', id: 'logdrum', type: 'folder',
    name: 'Log Drum Sessions', tag: 'amapiano', ts: 81, kindLabel: 'Folder', date: 'Yesterday, 20:14',
    items: [
      { key: 'file-ld-126', name: 'Log Drum 126.wav', kind: 'Audio', kindLabel: 'WAV audio', ext: 'WAV', icon: 'music-note', size: '12.8 MB', bytes: 12.8 * MB, date: 'Yesterday, 20:14', ts: 91, tag: 'amapiano' },
      { key: 'file-ld-128', name: 'Log Drum 128.wav', kind: 'Audio', kindLabel: 'WAV audio', ext: 'WAV', icon: 'music-note', size: '6.4 MB', bytes: 6.4 * MB, date: 'Yesterday, 20:12', ts: 90, tag: 'amapiano' },
      { key: 'file-ld-notes', name: 'Session Notes.txt', kind: 'Text', kindLabel: 'Plain text', ext: 'TXT', icon: 'text', size: '12 KB', bytes: 12 * KB, date: 'Yesterday, 20:09', ts: 89, tag: 'archive', content: 'Log drum sessions — 126 & 128 BPM\n\n01 Log Drum 126.wav   master take, 18.4 MB\n02 Shaker Groove.wav   loops, 6.2 MB\n\nNext: label the pack cover with the orbit artwork\nand keep the 126 BPM master for the release.' }
    ]
  },
  {
    key: 'fold-daw', id: 'daw', type: 'folder',
    name: 'SMART-DAW Beta', tag: 'design', ts: 79, kindLabel: 'Folder', date: 'Yesterday, 17:42',
    items: [
      { key: 'file-daw-app', name: 'smart-daw-0.4.2.AppImage', kind: 'App', kindLabel: 'Linux app', ext: 'APP', icon: 'widget', size: '22.4 MB', bytes: 22.4 * MB, date: 'Yesterday, 17:42', ts: 80, tag: 'design' },
      { key: 'file-daw-layout', name: 'Layout Study.png', kind: 'Image', kindLabel: 'PNG image', ext: 'PNG', icon: 'gallery', thumb: 'layout-study', dims: '2560 × 1600', size: '1.4 MB', bytes: 1.4 * MB, date: 'Yesterday, 14:22', ts: 85, tag: 'design' },
      { key: 'file-daw-notes', name: 'beta-notes.md', kind: 'Text', kindLabel: 'Markdown', ext: 'MD', icon: 'text', size: '12 KB', bytes: 12 * KB, date: 'Yesterday, 11:05', ts: 79, tag: 'design', content: '# smart-daw 0.4.2 beta\n\n- Timeline zoom now follows the playhead\n- Waveform cache writes to Smart Drive\n- Export panel is still a mock\n\nOwner: Naledzi · release target: pack volume 1' }
    ]
  },
  {
    key: 'fold-brand', id: 'brand', type: 'folder',
    name: 'Brand — Naledzi', tag: 'design', ts: 86, kindLabel: 'Folder', date: 'Oct 30, 19:36',
    items: [
      { key: 'file-brand-svg', name: 'Naledzi Wordmark.svg', kind: 'Vector', kindLabel: 'SVG vector', ext: 'SVG', icon: 'stars-minimalistic', size: '12 KB', bytes: 12 * KB, date: 'Oct 30, 19:36', ts: 88, tag: 'design', dims: '220 × 64', svg: '<svg viewBox="0 0 220 64" xmlns="http://www.w3.org/2000/svg"><defs><linearGradient id="nlg" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#2f6feb"/><stop offset="1" stop-color="#6f43c9"/></linearGradient></defs><circle cx="34" cy="32" r="17" fill="none" stroke="url(#nlg)" stroke-width="3"/><circle cx="34" cy="32" r="5" fill="url(#nlg)"/><text x="62" y="41" font-family="sans-serif" font-size="24" font-weight="600" letter-spacing="1.5" fill="#141d30">NALEDZI</text></svg>' },
      { key: 'file-brand-ref', name: 'Reference 01.jpg', kind: 'Image', kindLabel: 'JPEG image', ext: 'JPG', icon: 'gallery', thumb: 'reference-01', dims: '2400 × 1600', size: '1.6 MB', bytes: 1.6 * MB, date: 'Oct 30, 19:02', ts: 87, tag: 'design' },
      { key: 'file-brand-specimen', name: 'Type Specimen.pdf', kind: 'PDF', kindLabel: 'PDF document', ext: 'PDF', icon: 'document-text', size: '4.2 MB', bytes: 4.2 * MB, date: 'Oct 30, 18:48', ts: 86, tag: 'archive', content: 'NALEDZI — TYPE SPECIMEN\n\nDisplay  ·  Naledzi Display\nHeadline · 64 / -2 tracking\nBody     · Naledzi Sans 13\nMono     · Naledzi Mono 11\n\nAmapiano Pack Vol. 1 uses the frosted glass\ntexture and the orbit artwork as cover direction.' }
    ]
  }
];

const FILES = [
  { key: 'img-frosted', id: 'frosted', type: 'file', name: '4K Frosted Glass.jpg', kind: 'Image', kindLabel: 'JPEG image', ext: 'JPG', thumb: 'frosted-glass', dims: '3840 × 2160', size: '1.5 MB', bytes: 1.55 * MB, date: 'Today, 09:41', ts: 100, tag: 'design' },
  { key: 'img-brand-board', id: 'brand-board', type: 'file', name: 'Nixio Dashboard.png', kind: 'Image', kindLabel: 'PNG image', ext: 'PNG', thumb: 'brand-board', dims: '2400 × 1600', size: '1.7 MB', bytes: 1.7 * MB, date: 'Today, 08:12', ts: 96, tag: 'design' },
  { key: 'img-ui', id: 'ui', type: 'file', name: 'Window Study.png', kind: 'Image', kindLabel: 'PNG image', ext: 'PNG', thumb: 'ui-exploration', dims: '2880 × 1800', size: '878 KB', bytes: 878 * KB, date: 'Yesterday, 21:36', ts: 92, tag: 'design' },
  { key: 'img-scan', id: 'scan', type: 'file', name: 'Green Fold.png', kind: 'Image', kindLabel: 'PNG image', ext: 'PNG', thumb: 'design-scan', dims: '2560 × 1600', size: '868 KB', bytes: 868 * KB, date: 'Yesterday, 18:02', ts: 90, tag: 'design' },
  { key: 'img-package', id: 'package', type: 'file', name: 'Soweto Moto Ad.png', kind: 'Image', kindLabel: 'PNG image', ext: 'PNG', thumb: 'package-a', dims: '2000 × 1400', size: '1.1 MB', bytes: 1.1 * MB, date: 'Oct 30, 14:20', ts: 84, tag: 'amapiano' },
  { key: 'img-poster', id: 'poster', type: 'file', name: 'Orbit Poster.png', kind: 'Image', kindLabel: 'PNG image', ext: 'PNG', thumb: 'poster-01', dims: '2000 × 2500', size: '1.2 MB', bytes: 1.2 * MB, date: 'Oct 30, 11:04', ts: 82, tag: 'amapiano' },
  { key: 'img-talk', id: 'talk', type: 'file', name: 'Cycling in Soweto.png', kind: 'Image', kindLabel: 'PNG image', ext: 'PNG', thumb: 'talk-notes', dims: '2880 × 1800', size: '977 KB', bytes: 977 * KB, date: 'Oct 29, 16:48', ts: 74, tag: 'archive' },
  { key: 'img-texture', id: 'texture', type: 'file', name: 'Blossom Study.jpg', kind: 'Image', kindLabel: 'JPEG image', ext: 'JPG', thumb: 'texture-01', dims: '2400 × 1600', size: '812 KB', bytes: 812 * KB, date: 'Oct 28, 09:15', ts: 68, tag: 'archive' }
];

const DEVICES = [
  {
    id: 'drive', name: 'Smart Drive', kind: 'Built-in', icon: 'ssd-square',
    mounted: true, percent: 62, capacity: '194 GB free of 512 GB', tone: 'cobalt',
    files: []
  },
  {
    id: 'sanDisk', name: 'SanDisk Ultra', kind: 'USB 3.2', icon: 'ssd-square',
    mounted: false, percent: 36, capacity: '41 GB free of 64 GB', tone: 'amber',
    files: [
      { key: 'usb-flp', name: 'Amapiano Pack Vol. 1.flp', kind: 'Project', kindLabel: 'FL Studio project', ext: 'FLP', icon: 'file', size: '4.8 MB', bytes: 4.8 * MB, date: 'Today, 08:44', ts: 94, tag: 'amapiano' },
      { key: 'usb-cover', name: 'Pack Cover.jpg', kind: 'Image', kindLabel: 'JPEG image', ext: 'JPG', icon: 'gallery', thumb: 'poster-01', dims: '1600 × 1600', size: '2.1 MB', bytes: 2.1 * MB, date: 'Today, 08:40', ts: 93, tag: 'amapiano' },
      { key: 'usb-notes', name: 'Handover.txt', kind: 'Text', kindLabel: 'Plain text', ext: 'TXT', icon: 'text', size: '4 KB', bytes: 4000, date: 'Yesterday, 19:02', ts: 80, tag: 'archive', content: 'Staged for release:\n- Amapiano Pack Vol. 1.flp\n- Pack Cover.jpg\n\nCopy the 126 BPM master over before Friday.' }
    ],
    pack: []
  },
  {
    id: 'nas', name: 'Studio NAS', kind: 'Network', icon: 'global',
    mounted: true, percent: 74, capacity: '2.1 TB free of 8 TB', tone: 'green',
    files: [
      { key: 'nas-brand', name: 'Brand Master.zip', kind: 'Archive', kindLabel: 'ZIP archive', ext: 'ZIP', icon: 'archive', size: '248 MB', bytes: 248 * MB, date: 'Yesterday, 22:10', ts: 88, tag: 'design', contents: [{ name: 'brand-master.ai', size: '182 MB' }, { name: 'wordmark.svg', size: '12 KB' }, { name: 'palette.pdf', size: '1.4 MB' }, { name: 'README.txt', size: '2 KB' }, { name: 'icons/', size: '38 files' }] },
      { key: 'nas-spec', name: 'Type Specimen.pdf', kind: 'PDF', kindLabel: 'PDF document', ext: 'PDF', icon: 'document-text', size: '4.2 MB', bytes: 4.2 * MB, date: 'Oct 31, 10:12', ts: 70, tag: 'archive', content: 'NALEDZI — TYPE SPECIMEN\n\nShared from the Studio NAS.\nDisplay · Naledzi Display\nBody · Naledzi Sans 13' }
    ],
    backups: [
      { key: 'nas-backup-1', name: 'Session Backup 01.wav', kind: 'Audio', kindLabel: 'WAV audio', ext: 'WAV', icon: 'music-note', size: '18.4 MB', bytes: 18.4 * MB, date: 'Today, 07:58', ts: 95, tag: 'amapiano' }
    ]
  },
  {
    id: 'phone', name: 'Naledzi’s Pixel', kind: 'Continuity', icon: 'users-group-rounded',
    mounted: true, percent: 42, capacity: '42% battery · Wi-Fi', tone: 'cobalt',
    files: []
  }
];

const state = {
  route: { type: 'nav', id: 'pictures' },
  view: 'grid',
  sort: 'name',
  query: '',
  selection: new Set(),
  lastIndex: -1,
  trash: new Set(),
  clipboard: null,
  clipboardList: [],
  history: [{ type: 'nav', id: 'pictures' }],
  hIndex: 0,
  openModal: null,
  previewKey: null,
  aiOpen: false,
  paletteOpen: false,
  paletteRail: 'all',
  paletteQuery: '',
  paletteIndex: 0,
  paletteItems: [],
  paletteLastFocus: null,
  infoKey: null,
  shareKey: null,
  shareMode: 'share',
  wallpaper: 'frosted',
  audioElapsed: 0,
  lastMenuPos: { x: 0, y: 0 },
  opts: { ext: false, dates: true, density: 'comfortable' },
  devices: DEVICES.map(device => ({ ...device })),
  phoneFiles: [],
  documentsFiles: [],
  downloadsFiles: [],
  transfer: null,
  confirmAction: null
};

const $  = (sel, root = document) => root.querySelector(sel);
const $$ = (sel, root = document) => Array.from(root.querySelectorAll(sel));

function esc(value) {
  return String(value)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function fmt(bytes) {
  if (!bytes) return '0 KB';
  if (bytes >= 1e9) return (bytes / 1e9).toFixed(1) + ' GB';
  if (bytes >= 10 * MB) return Math.round(bytes / MB) + ' MB';
  if (bytes >= MB) return (bytes / MB).toFixed(1) + ' MB';
  return Math.max(1, Math.round(bytes / KB)) + ' KB';
}

function folderBytes(folder) {
  return folder.items.reduce((sum, item) => sum + (item.bytes || 0), 0);
}

function itemBytes(item) {
  return item.items ? folderBytes(item) : (item.bytes || 0);
}

function allFolderFiles() {
  return FOLDERS.flatMap(folder => folder.items);
}

function findItem(key) {
  const folder = FOLDERS.find(f => f.key === key);
  if (folder) return { item: folder, parent: null, isFolder: true };
  const file = FILES.find(f => f.key === key);
  if (file) return { item: file, parent: null, isFolder: false };
  for (const f of FOLDERS) {
    const inner = f.items.find(i => i.key === key);
    if (inner) return { item: inner, parent: f, isFolder: false };
  }
  for (const device of state.devices) {
    const lists = [device.files, device.backups, device.pack].filter(Boolean);
    for (const list of lists) {
      const file = list.find(i => i.key === key);
      if (file) return { item: file, parent: null, isFolder: false, device };
    }
  }
  for (const list of [state.phoneFiles, state.documentsFiles, state.downloadsFiles]) {
    const file = list.find(i => i.key === key);
    if (file) return { item: file, parent: null, isFolder: false };
  }
  return null;
}

function tagColor(tag) {
  return TAG[tag] ? TAG[tag].color : '#2f6feb';
}

/* Finder-style name: extensions stay hidden until the user asks for them. */
function displayName(item) {
  if (state.opts.ext || !item || item.items) return item.name;
  const dot = item.name.lastIndexOf('.');
  if (dot <= 0) return item.name;
  const ext = item.name.slice(dot + 1);
  if (!/^[A-Za-z0-9]{1,5}$/.test(ext)) return item.name;
  return item.name.slice(0, dot);
}

function sortList(list) {
  const arr = list.slice();
  const collator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });
  arr.sort((a, b) => {
    if (state.sort === 'kind') {
      return collator.compare(a.kindLabel || 'Folder', b.kindLabel || 'Folder') || collator.compare(a.name, b.name);
    }
    if (state.sort === 'date') return (b.ts || 0) - (a.ts || 0);
    if (state.sort === 'size') return itemBytes(b) - itemBytes(a);
    return collator.compare(a.name, b.name);
  });
  return arr;
}

function emptyState(view, q) {
  if (q) {
    return {
      icon: 'magnifer',
      title: `No matches for “${q}”`,
      copy: 'Try another name, or clear the search to see everything again.',
      cta: { label: 'Clear search', act: 'clear-search' }
    };
  }

  if (state.route.type === 'device') {
    const device = state.devices.find(entry => entry.id === state.route.id);
    if (device && !device.mounted) {
      return {
        icon: 'ssd-square',
        title: `${device.name} is not mounted`,
        copy: 'Mount it from Locations to browse its files again.',
        cta: { label: 'Mount device', act: 'mount-device' }
      };
    }
    if (device) {
      return {
        icon: 'ssd-square',
        title: `${device.name} is empty`,
        copy: 'Copy files here from Pictures, or open the device manager.',
        cta: { label: 'Open Pictures', act: 'go-pictures' }
      };
    }
  }

  if (view.title === 'Pictures') {
    return {
      icon: 'gallery',
      title: 'Pictures is empty',
      copy: 'Move something back from the Trash, or create a folder to get started.',
      cta: { label: 'New folder', act: 'new-folder' }
    };
  }

  if (state.route.type === 'folder') {
    return {
      icon: 'folder',
      title: 'This folder is empty',
      copy: 'Drop files here or create something new. This is your space.',
      cta: { label: 'Open Pictures', act: 'go-pictures' }
    };
  }

  if (state.route.type === 'tag') {
    return {
      icon: 'tag',
      title: `No ${view.title} items`,
      copy: 'Tag files from the context menu and they will collect here.',
      cta: { label: 'Open Pictures', act: 'go-pictures' }
    };
  }

  const map = {
    home:      { icon: 'home', title: 'Home is quiet', copy: 'Files you save to Home will show up here.', cta: { label: 'Open Pictures', act: 'go-pictures' } },
    recents:   { icon: 'clock-circle', title: 'Nothing recent', copy: 'Open a few files and they will appear in this view.', cta: { label: 'Open Pictures', act: 'go-pictures' } },
    desktop:   { icon: 'monitor', title: 'Desktop is clear', copy: 'Drop files here and they will show up instantly.', cta: { label: 'Open Pictures', act: 'go-pictures' } },
    documents: { icon: 'document-text', title: 'No documents yet', copy: 'Project files and notes will gather here.', cta: { label: 'Open Pictures', act: 'go-pictures' } },
    downloads: { icon: 'download-minimalistic', title: 'Downloads is empty', copy: 'Everything has been filed away. New downloads land here.', cta: { label: 'Open Pictures', act: 'go-pictures' } },
    music:     { icon: 'music-note', title: 'No music yet', copy: 'Audio from your sessions will appear here.', cta: { label: 'Open Pictures', act: 'go-pictures' } },
    movies:    { icon: 'clapperboard', title: 'No movies yet', copy: 'Video work will appear here when you bring it in.', cta: { label: 'Open Pictures', act: 'go-pictures' } },
    network:   { icon: 'global', title: 'No network locations', copy: 'Shared drives will appear here when you connect them.', cta: { label: 'Open Pictures', act: 'go-pictures' } },
    cloud:     { icon: 'cloud', title: 'Smart Cloud is up to date', copy: 'Everything in Smart Drive is synced. Nothing waiting.', cta: { label: 'Open Smart Drive', act: 'go-drive' } },
    drive:     { icon: 'ssd-square', title: 'Smart Drive is empty', copy: 'Your files will live here, across every Smart app.', cta: { label: 'Open Pictures', act: 'go-pictures' } }
  };

  return map[state.route.id] || {
    icon: 'folder',
    title: 'Nothing here yet',
    copy: 'This view will fill up as you use Smart Files.',
    cta: { label: 'Open Pictures', act: 'go-pictures' }
  };
}

function getView() {
  const route = state.route;
  let folders = [];
  let items = [];
  let title = 'Pictures';
  let icon = 'gallery';
  let section = 'Items';

  if (state.route.type === 'device') {
    const device = state.devices.find(entry => entry.id === state.route.id);
    if (device) {
      title = device.name;
      icon = device.icon;
      items = device.mounted ? (device.files || []).slice() : [];
      section = device.mounted ? 'Contents' : 'Ejected';
    }
  } else if (route.type === 'folder') {
    const folder = FOLDERS.find(f => f.id === route.id);
    if (folder) {
      title = folder.name;
      icon = 'folder';
      items = folder.items.slice();
      section = 'Contents';
    }
  } else if (route.type === 'tag') {
    title = TAG[route.id] ? TAG[route.id].label : 'Tag';
    icon = 'tag';
    items = [...FILES, ...allFolderFiles()].filter(f => f.tag === route.id);
    section = 'Tagged ' + title;
  } else {
    const nav = NAV[route.id] || NAV.pictures;
    title = nav.title;
    icon = nav.icon;
    switch (route.id) {
      case 'pictures':
        folders = FOLDERS.slice();
        items = FILES.slice();
        section = 'Images';
        break;
      case 'drive':
        folders = [];
        items = [...FILES, ...allFolderFiles()];
        section = 'All files';
        break;
      case 'recents':
        items = [...FILES, ...allFolderFiles()].sort((a, b) => (b.ts || 0) - (a.ts || 0)).slice(0, 9);
        section = 'Recently modified';
        break;
      case 'music':
        items = allFolderFiles().filter(i => i.kind === 'Audio');
        section = 'Tracks';
        break;
      case 'documents':
        items = allFolderFiles().filter(i => ['Project', 'Text', 'PDF', 'Vector'].includes(i.kind));
        section = 'Documents';
        break;
      default:
        break;
    }
  }

  folders = folders.filter(f => !state.trash.has(f.key));
  items = items.filter(i => !state.trash.has(i.key) && !state.trash.has(i.parentKey || ''));

  const q = state.query.trim().toLowerCase();
  if (q) {
    folders = folders.filter(f => f.name.toLowerCase().includes(q));
    items = items.filter(i => i.name.toLowerCase().includes(q));
  }

  folders = sortList(folders);
  items = sortList(items);

  const bytes = folders.reduce((sum, f) => sum + folderBytes(f), 0)
              + items.reduce((sum, i) => sum + (i.bytes || 0), 0);

  const view = {
    route,
    title,
    icon,
    section,
    folders,
    items,
    count: folders.length + items.length,
    bytes,
    query: q
  };

  view.empty = emptyState(view, state.query.trim());
  return view;
}

/* ============================================================
   icons · chrome · body
   ============================================================ */

function hydrateIcons(root = document) {
  $$('[data-ic]', root).forEach(el => {
    if (el.dataset.icDone) return;
    const name = el.dataset.ic;
    el.innerHTML =
      `<svg class="o" aria-hidden="true"><use href="#i-${name}-outline"></use></svg>` +
      `<svg class="d" aria-hidden="true"><use href="#i-${name}-duotone"></use></svg>` +
      `<svg class="f" aria-hidden="true"><use href="#i-${name}-filled"></use></svg>`;
    el.dataset.icDone = '1';
    el.setAttribute('aria-hidden', 'true');
  });
}

const SORT_LABEL = { name: 'Name', kind: 'Kind', date: 'Modified', size: 'Size' };

function plural(n, one, many) {
  return n === 1 ? one : many;
}

function crumb(label, token, current = false, extra = '') {
  if (current) {
    return `<button class="crumb current ${extra}" aria-current="page">${esc(label)}</button>`;
  }
  return `<button class="crumb ${extra}" data-goto="${token}">${esc(label)}</button>`;
}

function renderCrumbs(view) {
  const route = view.route;
  let html = `<button class="crumb root" data-goto="drive">Smart</button><span class="csep root">›</span>`;

  if (route.type === 'folder') {
    html += crumb('Pictures', 'pictures');
    html += '<span class="csep">›</span>';
    html += crumb(view.title, '', true, 'trunc');
  } else if (route.type === 'tag') {
    html += crumb('Pictures', 'pictures');
    html += '<span class="csep">›</span>';
    html += crumb(view.title, '', true);
  } else if (route.id === 'pictures') {
    html += crumb('Pictures', '', true);
  } else {
    html += crumb(view.title, '', true);
  }

  $('#crumbs').innerHTML = html;
  hydrateIcons($('#crumbs'));
}

function renderChrome(view) {
  $('#pageTitle').textContent = view.title;

  let sub = `<b>${view.count} ${plural(view.count, 'item', 'items')}</b><span class="dotsep"></span>${fmt(view.bytes)}`;
  if (view.folders.length) {
    sub += `<span class="dotsep hide-sm"></span><span class="hide-sm">${view.folders.length} ${plural(view.folders.length, 'folder', 'folders')}</span>`;
  }
  if (view.query) {
    sub += `<span class="dotsep"></span><span>filtered</span>`;
  }
  $('#pageSub').innerHTML = sub;

  renderCrumbs(view);

  $('#searchInput').placeholder = `Search ${view.title.length > 11 ? 'folder' : view.title}`;

  const sortLabel = $('#sortLabel');
  if (sortLabel) sortLabel.textContent = SORT_LABEL[state.sort] || 'Name';

  $('#gridBtn').classList.toggle('on', state.view === 'grid');
  $('#listBtn').classList.toggle('on', state.view === 'list');
  $('#gridBtn').setAttribute('aria-pressed', String(state.view === 'grid'));
  $('#listBtn').setAttribute('aria-pressed', String(state.view === 'list'));

  $('#backBtn').disabled = state.hIndex <= 0;
  $('#fwdBtn').disabled = state.hIndex >= state.history.length - 1;

  const route = view.route;
  const navKey = route.type === 'nav' ? route.id : (route.type === 'tag' ? `tag:${route.id}` : '');
  $$('#sideNav .row').forEach(row => {
    row.classList.toggle('active', row.dataset.nav === navKey && !!navKey);
  });

  const visibleTop = FOLDERS.filter(f => !state.trash.has(f.key)).length + FILES.filter(f => !state.trash.has(f.key)).length;
  const tail = $('.row[data-nav="pictures"] .tail');
  if (tail) tail.textContent = visibleTop;

  Object.keys(TAG).forEach(id => {
    const tagTail = $(`.row[data-nav="tag:${id}"] .tail`);
    if (tagTail) {
      const count = [...FILES, ...allFolderFiles()].filter(file =>
        file.tag === id
        && !state.trash.has(file.key)
        && !state.trash.has(file.parentKey || '')
      ).length;
      tagTail.textContent = count;
    }
  });

  renderSidebarDevices();

  document.title = `${view.title} — Smart`;
}

function sectionHead(title, count) {
  return `<div class="section-head"><span class="section-title">${esc(title)}</span><span class="section-count">${count}</span><span class="section-line"></span></div>`;
}

function folderCard(folder) {
  const bytes = folderBytes(folder);
  return `<article class="folder" data-key="${folder.key}" tabindex="0" role="button" aria-label="${esc(folder.name)}, ${folder.items.length} items">
    <span class="ftile"><span class="ic ic-duo" data-ic="folder"></span></span>
    <span class="fmeta">
      <span class="fname">${esc(folder.name)}</span>
      <span class="fsub">${folder.items.length} ${plural(folder.items.length, 'item', 'items')} · ${fmt(bytes)}</span>
    </span>
    <span class="fdot" style="--c:${tagColor(folder.tag)}" aria-hidden="true"></span>
    <button class="fmore" data-act="menu" aria-label="More for ${esc(folder.name)}"><span class="ic" data-ic="menu-dots"></span></button>
  </article>`;
}

function waveMarkup(key) {
  let seed = 7;
  for (let i = 0; i < key.length; i += 1) seed = (seed * 31 + key.charCodeAt(i)) % 9973;
  const bars = [];
  for (let i = 0; i < 52; i += 1) {
    const a = Math.sin((i + seed % 11) * 0.52) * 0.5 + 0.5;
    const b = Math.sin((i * 1.73 + seed % 17) * 0.41) * 0.5 + 0.5;
    const h = 9 + Math.round((a * 0.62 + b * 0.38) * 40);
    const y = Math.round(32 - h / 2);
    bars.push(`<rect x="${i * 4}" y="${y}" width="2" height="${h}" rx="1"/>`);
  }
  return `<svg class="wave" viewBox="0 0 208 64" preserveAspectRatio="none" aria-hidden="true"><g fill="currentColor">${bars.join('')}</g></svg>`;
}

function card(item) {
  const isImage = !!item.thumb;
  const isAudio = item.kind === 'Audio';
  const tag = item.tag ? `<span class="ctag" style="--c:${tagColor(item.tag)}" aria-hidden="true"></span>` : '';

  const media = isImage
    ? `<img src="assets/thumbs/${item.thumb}.jpg" alt="${esc(item.name)}" draggable="false">`
    : isAudio
      ? `<span class="wavewrap">${waveMarkup(item.key)}</span>
         <span class="acap"><span class="ic" data-ic="music-note"></span>${esc(item.size)}</span>
         <span class="ext">${esc(item.ext || 'WAV')}</span>`
      : `<span class="ic ic-duo genicon" data-ic="${item.icon || 'file'}"></span><span class="gendim">${esc(item.kindLabel || 'File')}</span><span class="ext">${esc(item.ext || 'FILE')}</span>`;

  const quick = isImage
    ? `<button class="qbtn" data-act="preview" aria-label="Preview ${esc(item.name)}"><span class="ic" data-ic="eye"></span></button>
       <button class="qbtn" data-act="menu" aria-label="Actions for ${esc(item.name)}"><span class="ic" data-ic="menu-dots"></span></button>`
    : `<button class="qbtn" data-act="open" aria-label="Open ${esc(item.name)}"><span class="ic" data-ic="arrow-right-up"></span></button>
       <button class="qbtn" data-act="menu" aria-label="Actions for ${esc(item.name)}"><span class="ic" data-ic="menu-dots"></span></button>`;

  const meta = isImage
    ? `${esc(item.size)}${state.opts.dates ? `<span class="dotsep"></span>${esc(item.date)}` : ''}`
    : `${esc(item.kindLabel || 'File')}<span class="dotsep"></span>${esc(item.size)}${state.opts.dates ? `<span class="dotsep"></span>${esc(item.date)}` : ''}`;

  return `<article class="card" data-key="${item.key}" tabindex="0" role="button" aria-label="${esc(item.name)}">
    <div class="thumb ${isImage ? '' : 'gen'}">
      ${media}
      <span class="selcheck"><span class="ic" data-ic="check-circle"></span></span>
      <div class="quick">${quick}</div>
    </div>
    <div class="cinfo">
      <div class="cinfo-main">
        <div class="cname"><span>${esc(displayName(item))}</span>${tag}</div>
        <div class="cmeta">${meta}</div>
      </div>
    </div>
  </article>`;
}

function listRow(entry) {
  const isFolder = !!entry.items;
  const item = entry;
  const media = isFolder
    ? `<span class="licon"><span class="ic ic-duo" data-ic="folder"></span></span>`
    : item.thumb
      ? `<span class="licon"><img src="assets/thumbs/${item.thumb}.jpg" alt="" draggable="false"></span>`
      : `<span class="licon"><span class="ic" data-ic="${item.icon || 'file'}"></span></span>`;

  const tag = item.tag ? `<span class="ctag" style="--c:${tagColor(item.tag)}" aria-hidden="true"></span>` : '';
  const size = isFolder ? fmt(folderBytes(item)) : item.size;
  const kind = isFolder ? 'Folder' : (item.kindLabel || 'File');
  const date = state.opts.dates ? (item.date || 'Today') : '—';

  return `<div class="lrow" data-key="${item.key}" tabindex="0" role="button" aria-label="${esc(item.name)}">
    ${media}
    <div class="lmain"><span class="lname">${esc(displayName(item))}</span>${tag}</div>
    <div class="lcell lkind-cell">${esc(kind)}</div>
    <div class="lcell lsize">${esc(size)}</div>
    <div class="lcell">${esc(date)}</div>
    <button class="lmore" data-act="menu" aria-label="Actions for ${esc(item.name)}"><span class="ic" data-ic="menu-dots"></span></button>
  </div>`;
}

function emptyMarkup(view) {
  const empty = view.empty;
  const icons = { 'clear-search': 'close-circle', 'new-folder': 'add-folder', 'go-pictures': 'gallery', 'go-drive': 'ssd-square' };
  const icon = icons[empty.cta.act] || 'folder';
  return `<div class="empty">
    <span class="empty-orb"><span class="ic" data-ic="${empty.icon}"></span></span>
    <h3>${esc(empty.title)}</h3>
    <p>${esc(empty.copy)}</p>
    <button class="pill-action" data-empty-act="${empty.cta.act}"><span class="ic" data-ic="${icon}"></span>${esc(empty.cta.label)}</button>
  </div>`;
}

function renderBody(view) {
  const mount = $('#viewMount');

  if (!view.count) {
    mount.innerHTML = emptyMarkup(view);
    hydrateIcons(mount);
    return;
  }

  if (state.view === 'list') {
    const rows = [...view.folders.map(listRow), ...view.items.map(listRow)].join('');
    mount.innerHTML = sectionHead('All items', view.count) + `<div class="list">
      <div class="list-head">
        <span></span>
        <button data-sort="name">Name</button>
        <button class="lkind-col" data-sort="kind">Kind</button>
        <button data-sort="size">Size</button>
        <button data-sort="date">Modified</button>
        <span></span>
      </div>
      <div class="list-body">${rows}</div>
    </div>`;
    hydrateIcons(mount);
    return;
  }

  let html = '';
  if (view.folders.length) {
    html += sectionHead('Folders', view.folders.length);
    html += `<div class="folders">${view.folders.map(folderCard).join('')}</div>`;
  }
  if (view.items.length) {
    const title = view.section || 'Items';
    const density = state.opts.density === 'compact' ? ' compact' : '';
    html += sectionHead(title, view.items.length);
    html += `<div class="grid${density}">${view.items.map(card).join('')}</div>`;
  }
  mount.innerHTML = html;
  hydrateIcons(mount);
}

function updateStatus(view) {
  const selected = [...state.selection].map(key => findItem(key)?.item).filter(Boolean);
  const left = $('#statusLeft');

  if (selected.length) {
    const bytes = selected.reduce((sum, item) => sum + itemBytes(item), 0);
    left.innerHTML = `<b>${selected.length} selected</b><span class="status-dot"></span>${fmt(bytes)}<span class="status-dot"></span>of ${view.count} ${plural(view.count, 'item', 'items')}`;
  } else {
    left.innerHTML = `<b>${view.count} ${plural(view.count, 'item', 'items')}</b><span class="status-dot"></span>${fmt(view.bytes)}`;
  }
}

function refreshSelection() {
  $$('[data-key]').forEach(el => {
    el.classList.toggle('sel', state.selection.has(el.dataset.key));
  });
  updateStatus(getView());
}

function render() {
  const view = getView();
  renderChrome(view);
  renderBody(view);
  updateStatus(view);
  hydrateIcons(document);
  refreshSelection();
}

/* ============================================================
   navigation · selection · preview
   ============================================================ */

function routeToken(route) {
  if (route.type === 'folder') return `folder:${route.id}`;
  if (route.type === 'tag') return `tag:${route.id}`;
  return route.id;
}

function tokenRoute(token) {
  if (token.startsWith('folder:')) return { type: 'folder', id: token.slice(7) };
  if (token.startsWith('tag:')) return { type: 'tag', id: token.slice(4) };
  return { type: 'nav', id: token };
}

function sameRoute(a, b) {
  return a.type === b.type && (a.id || '') === (b.id || '');
}

function applyRoute(route) {
  state.route = { ...route };
  state.selection.clear();
  state.lastIndex = -1;
  state.query = '';
  const input = $('#searchInput');
  input.value = '';
  $('#searchBox').classList.remove('has-val');
  render();
  $('#content').scrollTop = 0;
}

function navigate(route, push = true) {
  if (push && sameRoute(state.route, route)) {
    state.query = '';
    $('#searchInput').value = '';
    $('#searchBox').classList.remove('has-val');
    clearSelection();
    return;
  }
  if (push) {
    state.history = state.history.slice(0, state.hIndex + 1);
    state.history.push({ ...route });
    state.hIndex = state.history.length - 1;
  }
  applyRoute(route);
}

function goBack() {
  if (state.hIndex <= 0) return;
  state.hIndex -= 1;
  applyRoute(state.history[state.hIndex]);
}

function goForward() {
  if (state.hIndex >= state.history.length - 1) return;
  state.hIndex += 1;
  applyRoute(state.history[state.hIndex]);
}

function currentKeys() {
  const view = getView();
  return [...view.folders.map(f => f.key), ...view.items.map(i => i.key)];
}

function selectKey(key, event = {}) {
  const keys = currentKeys();
  const index = keys.indexOf(key);
  if (index < 0) return;

  if (event.shiftKey && state.lastIndex >= 0 && state.lastIndex < keys.length) {
    const lo = Math.min(state.lastIndex, index);
    const hi = Math.max(state.lastIndex, index);
    for (let i = lo; i <= hi; i += 1) state.selection.add(keys[i]);
  } else if (event.metaKey || event.ctrlKey) {
    if (state.selection.has(key)) state.selection.delete(key);
    else state.selection.add(key);
  } else {
    state.selection.clear();
    state.selection.add(key);
  }
  state.lastIndex = index;
  refreshSelection();
}

function clearSelection() {
  state.selection.clear();
  state.lastIndex = -1;
  refreshSelection();
}

function selectAll() {
  currentKeys().forEach(key => state.selection.add(key));
  state.lastIndex = currentKeys().length - 1;
  refreshSelection();
}

function singleSelected() {
  return state.selection.size === 1 ? [...state.selection][0] : null;
}

function openInApp(item) {
  const app = item.kind === 'Archive' ? 'Smart Archive Manager'
    : item.kind === 'Vector' ? 'Smart Vector'
    : 'Smart Files';
  toast(`Opening in ${app}…`, null, null, 'arrow-right-up');
}

function openKey(key) {
  const found = findItem(key);
  if (!found) return;
  const item = found.item;
  if (item.items) {
    navigate({ type: 'folder', id: item.id });
    return;
  }
  if (previewKind(item)) {
    openPreview(key);
    return;
  }
  openInApp(item);
}

function previewList() {
  return getView().items.filter(i => previewKind(i));
}

function updatePreview() {
  const found = findItem(state.previewKey);
  if (!found) return;
  const item = found.item;
  const kind = previewKind(item);
  if (!kind) return;
  const list = previewList();

  const image = $('#pvImg');
  const stage = $('#pvStage');
  if (kind === 'image') {
    image.src = `assets/thumbs/${item.thumb}.jpg`;
    image.alt = item.name;
    image.hidden = false;
    stage.hidden = true;
    stage.className = 'pv-stage';
    stage.innerHTML = '';
  } else {
    image.hidden = true;
    stage.hidden = false;
    stage.className = `pv-stage ${kind}`;
    stage.innerHTML = kind === 'audio'
      ? audioStageMarkup(item)
      : kind === 'doc'
        ? docStageMarkup(item)
        : kind === 'vector'
          ? vectorStageMarkup(item)
          : kind === 'archive'
            ? archiveStageMarkup(item)
            : appStageMarkup(item);
    hydrateIcons(stage);
    if (kind === 'audio') updateAudioUI(state.audioElapsed || 0, audioDuration(item));
  }

  const bits = [item.kindLabel || 'File'];
  if (kind === 'image') bits.push(item.dims || '—');
  if (kind === 'audio') bits.push(fmtTime(audioDuration(item)));
  if (kind === 'app') bits.push('Linux');
  if (kind === 'vector') bits.push('Scalable');
  if (kind === 'archive') bits.push(`${(item.contents || []).length} entries`);
  bits.push(item.size || '—', item.date || '');
  $('#pvName').textContent = item.name;
  $('#pvMeta').innerHTML = bits.map((bit, index) => (index ? '<span class="dotsep"></span>' : '') + esc(bit)).join('');

  const open = $('#pvOpen');
  open.innerHTML = kind === 'image'
    ? '<span class="ic" data-ic="arrow-right-up"></span>Open with…'
    : kind === 'audio'
      ? '<span class="ic" data-ic="music-notes"></span>Open in DAW'
      : kind === 'doc'
        ? '<span class="ic" data-ic="document-text"></span>Open in Reader'
        : kind === 'vector'
          ? '<span class="ic" data-ic="stars-minimalistic"></span>Open in Vector'
          : kind === 'archive'
            ? '<span class="ic" data-ic="archive"></span>Open Archive'
            : '<span class="ic" data-ic="widget"></span>Launch';
  hydrateIcons(open);

  const many = list.length > 1;
  $('#pvPrev').style.visibility = many ? 'visible' : 'hidden';
  $('#pvNext').style.visibility = many ? 'visible' : 'hidden';
  $('#pvPrev').disabled = !many;
  $('#pvNext').disabled = !many;
}

function openPreview(key) {
  const found = findItem(key);
  if (!found || !previewKind(found.item)) {
    if (found) openInfo(key);
    return;
  }
  stopAudioPreview();
  state.previewKey = key;
  updatePreview();
  openModal('preview');
}

function stepPreview(direction) {
  const list = previewList();
  if (list.length < 2) return;
  let index = list.findIndex(i => i.key === state.previewKey);
  index = (index + direction + list.length) % list.length;
  state.previewKey = list[index].key;
  updatePreview();
}

/* ============================================================
   overlays · toast · theme · sidebar
   ============================================================ */

let toastTimer = null;

function toast(message, actionLabel = null, actionFn = null, icon = 'check-circle') {
  const el = $('#toast');
  el.innerHTML = `<span class="ic" data-ic="${icon}"></span><span>${esc(message)}</span>`
    + (actionLabel ? `<button class="toast-action">${esc(actionLabel)}</button>` : '');
  hydrateIcons(el);
  el.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => el.classList.remove('show'), 3000);
  if (actionLabel && actionFn) {
    const action = el.querySelector('.toast-action');
    if (action) {
      action.addEventListener('click', event => {
        event.stopPropagation();
        el.classList.remove('show');
        actionFn();
      });
    }
  }
}

function openModal(name) {
  closeMenus();
  closeAI();
  closePalette(false);
  state.openModal = name;
  $$('.modal').forEach(modal => modal.classList.toggle('show', modal.id === name));
  $('#scrim').classList.add('show');
  if (name === 'tokens') renderTokens();
}

function closeModals() {
  state.openModal = null;
  state.previewKey = null;
  stopAudioPreview();
  stopAppPreview();
  $('#scrim').classList.remove('show');
  $$('.modal').forEach(modal => modal.classList.remove('show'));
}

function openAI() {
  closeModals();
  closeMenus();
  closePalette(false);
  state.aiOpen = true;
  $('#aiPanel').classList.add('show');
  $('#askBtn').setAttribute('aria-expanded', 'true');
}

function closeAI() {
  state.aiOpen = false;
  $('#aiPanel').classList.remove('show');
  $('#askBtn').setAttribute('aria-expanded', 'false');
}

function toggleAI() {
  if (state.aiOpen) closeAI();
  else openAI();
}

function setCollapsed(collapsed) {
  const win = $('#win');
  win.classList.toggle('is-collapsed', collapsed);
  const tip = collapsed ? 'Show sidebar  ⌘\\' : 'Hide sidebar  ⌘\\';
  const label = collapsed ? 'Show sidebar' : 'Hide sidebar';
  $('#sbToggle').setAttribute('data-tip', tip);
  $('#sbToggle').setAttribute('aria-label', label);
  $('#sbCollapse').setAttribute('aria-label', 'Hide sidebar');
}

function toggleSidebar() {
  setCollapsed(!$('#win').classList.contains('is-collapsed'));
}

function setTheme(theme) {
  document.documentElement.dataset.theme = theme;
}

function toggleTheme() {
  const next = document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark';
  setTheme(next);
  toast(next === 'dark' ? 'Dark appearance' : 'Light appearance', null, null, 'stars-minimalistic');
}

function gridOn() {
  return $('#gridOverlay').classList.contains('on');
}

function toggleGrid() {
  const overlay = $('#gridOverlay');
  overlay.classList.toggle('on');
  toast(overlay.classList.contains('on') ? '8 px grid overlay on' : 'Grid overlay off', null, null, 'widget');
}

/* ============================================================
   command palette · rail · stream · context
   ============================================================ */

const PALETTE_RAILS = ['all', 'go', 'files', 'make', 'look'];
const PALETTE_GROUP_ORDER = ['Navigate', 'Files', 'Create', 'Look', 'Ask Smart'];
const PALETTE_RAIL_LABEL = { all: 'All', go: 'Go', files: 'Files', make: 'Make', look: 'Look' };

function paletteFileCommand(key) {
  const found = findItem(key);
  if (!found) return null;
  const item = found.item;
  const kind = previewKind(item);
  if (!kind) return null;
  const icon = item.thumb ? 'gallery' : (item.icon || 'file');
  const action = kind === 'app' ? 'Launch' : 'Quick Look';
  return {
    id: `file-${key}`,
    rail: 'files',
    group: 'Files',
    icon,
    title: `${action} · ${item.name}`,
    sub: `${item.kindLabel} · ${item.size} · ${item.date}`,
    keywords: `${item.name} ${item.kindLabel} ${item.ext || ''} ${item.tag || ''} ${kind}`,
    run: () => openPreview(key),
    detail: {
      eyebrow: kind === 'app' ? 'Launch' : 'Quick look',
      title: item.name,
      copy: kind === 'app'
        ? `Start ${item.name} and watch it come up — launch progress stays inside the preview.`
        : `Open a full-bleed preview of ${item.name} without leaving the command surface.`,
      chips: [item.kindLabel, item.dims, item.size].filter(Boolean),
      thumb: item.thumb,
      shotName: item.name,
      tag: TAG[item.tag] ? TAG[item.tag].label : ''
    }
  };
}

function paletteCommands() {
  const cmds = [];
  const add = command => { if (command) cmds.push(command); };
  const view = getView();

  add({
    id: 'nav-pictures', rail: 'go', group: 'Navigate', icon: 'gallery',
    title: 'Open Pictures', sub: '8 images · 4 folders · 89 MB',
    keywords: 'pictures images gallery photography folders artwork',
    run: () => navigate({ type: 'nav', id: 'pictures' }),
    detail: {
      eyebrow: 'Destination', title: 'Pictures',
      copy: 'The visual library for this workspace — frosted glass, brand boards, and the first Amapiano artwork.',
      chips: ['8 images', '4 folders', '89 MB'],
      thumbs: ['frosted-glass', 'brand-board', 'poster-01']
    }
  });
  add({
    id: 'nav-drive', rail: 'go', group: 'Navigate', icon: 'ssd-square',
    title: 'Open Smart Drive', sub: 'Every file, across every Smart app',
    keywords: 'drive storage ssd all files smart cloud',
    run: () => navigate({ type: 'nav', id: 'drive' }),
    detail: {
      eyebrow: 'Destination', title: 'Smart Drive',
      copy: 'One index across Pictures, music sessions, brand work, and the DAW beta. 62% of 512 GB used.',
      chips: ['12 items', '89 MB', '62% used'],
      thumbs: ['brand-board', 'poster-01', 'frosted-glass']
    }
  });
  add({
    id: 'nav-recents', rail: 'go', group: 'Navigate', icon: 'clock-circle',
    title: 'Open Recents', sub: 'Everything you touched this week',
    keywords: 'recent history clock last opened',
    run: () => navigate({ type: 'nav', id: 'recents' }),
    detail: {
      eyebrow: 'Destination', title: 'Recents', icon: 'clock-circle',
      copy: 'A chronological thread of the work in motion. Six files and two folders changed today.',
      chips: ['Today', '6 files', '2 folders'],
      stageMeta: ['Newest', 'Today 09:41']
    }
  });
  add({
    id: 'nav-documents', rail: 'go', group: 'Navigate', icon: 'document-text',
    title: 'Open Documents', sub: 'Project files and notes',
    keywords: 'documents text notes pdf project',
    run: () => navigate({ type: 'nav', id: 'documents' }),
    detail: {
      eyebrow: 'Destination', title: 'Documents', icon: 'document-text', tone: 'var(--good)',
      copy: 'Project notes, specs, and the writing that keeps the suite honest.',
      chips: ['4 documents', '8.2 MB'],
      stageMeta: ['Notes', 'Synced']
    }
  });
  add({
    id: 'tag-design', rail: 'go', group: 'Navigate', icon: 'tag',
    title: 'Open Design tag', sub: '9 files across 4 folders',
    keywords: 'tag design brand ui artwork',
    run: () => navigate({ type: 'tag', id: 'design' }),
    detail: {
      eyebrow: 'Tag', title: 'Design', icon: 'tag',
      copy: 'A global thread through the workspace. Tags collect files inside folders, not just the surface.',
      chips: ['9 files', 'Across 4 folders', 'Global'],
      stageMeta: ['Nixio', 'Smart shell']
    }
  });
  add({
    id: 'tag-amapiano', rail: 'go', group: 'Navigate', icon: 'tag',
    title: 'Open Amapiano tag', sub: '8 files across 3 folders',
    keywords: 'tag amapiano music log drum artwork',
    run: () => navigate({ type: 'tag', id: 'amapiano' }),
    detail: {
      eyebrow: 'Tag', title: 'Amapiano', icon: 'tag', tone: 'var(--warn)',
      copy: 'Log drums, shakers, project files, and the artwork that carries the sound.',
      chips: ['8 files', 'Across 3 folders', 'Global'],
      stageMeta: ['126 BPM', 'Log drum']
    }
  });
  add({
    id: 'folder-amapiano', rail: 'go', group: 'Navigate', icon: 'folder',
    title: 'Open Amapiano Pack Vol. 1', sub: '4 items · 31.5 MB',
    keywords: 'folder amapiano pack audio wav flp cover',
    run: () => navigate({ type: 'folder', id: 'amapiano' }),
    detail: {
      eyebrow: 'Folder', title: 'Amapiano Pack Vol. 1', icon: 'folder', tone: 'var(--warn)',
      copy: 'Two WAV loops, the FL Studio project, and the pack cover. The first release-shaped folder.',
      chips: ['4 items', '31.5 MB', 'Audio + art'],
      stageMeta: ['126 BPM', '4 items']
    }
  });
  add({
    id: 'folder-brand', rail: 'go', group: 'Navigate', icon: 'folder',
    title: 'Open Brand — Naledzi', sub: '3 files · 5.8 MB',
    keywords: 'folder brand naledzi wordmark reference specimen',
    run: () => navigate({ type: 'folder', id: 'brand' }),
    detail: {
      eyebrow: 'Folder', title: 'Brand — Naledzi',
      copy: 'Wordmark, a photographic reference, and the type specimen. The identity in three artifacts.',
      chips: ['3 files', '5.8 MB', 'Identity'],
      thumbs: ['brand-board', 'ui-exploration', 'reference-01']
    }
  });

  add(paletteFileCommand('img-frosted'));
  add(paletteFileCommand('img-brand-board'));
  add(paletteFileCommand('img-ui'));
  add(paletteFileCommand('img-poster'));
  add(paletteFileCommand('file-logdrum-126'));
  add(paletteFileCommand('file-ld-notes'));
  add(paletteFileCommand('file-brand-specimen'));
  add(paletteFileCommand('file-daw-app'));
  add({
    id: 'devices', rail: 'go', group: 'Navigate', icon: 'ssd-square',
    title: 'Devices & locations', sub: `${state.devices.filter(d => d.mounted).length} mounted · ${state.devices.length} known`,
    keywords: 'devices locations mount eject drive usb nas hardware storage',
    run: () => openDevices(),
    detail: {
      eyebrow: 'Hardware', title: 'Devices & locations', icon: 'ssd-square',
      copy: 'Mount and eject storage without leaving the app. Ejected devices disappear from the sidebar until you bring them back.',
      chips: ['Smart Drive', 'SanDisk Ultra', 'Studio NAS'],
      stageMeta: ['Mount', 'Eject']
    }
  });
  add({
    id: 'phone', rail: 'go', group: 'Navigate', icon: 'users-group-rounded',
    title: 'Phone continuity', sub: 'Naledzi’s Pixel · paired over Wi-Fi',
    keywords: 'phone pixel continuity handoff clipboard relay mobile',
    run: () => openPhone(),
    detail: {
      eyebrow: 'Continuity', title: 'Naledzi’s Pixel', icon: 'users-group-rounded', tone: 'var(--good)',
      copy: 'Universal clipboard, hand-off of the current selection, and text relay — all over your local network.',
      chips: ['42% battery', 'Wi-Fi', 'Local only'],
      stageMeta: ['Paired', 'Wi-Fi']
    }
  });
  add({
    id: 'info-selection', rail: 'files', group: 'Files', icon: 'info-circle',
    title: 'Get Info on the selection', sub: 'Kind, size, tags, dates, location',
    keywords: 'info details properties metadata size kind get info',
    run: () => {
      const selected = singleSelected();
      if (selected) openInfo(selected);
      else openLocationInfo();
    },
    detail: {
      eyebrow: 'Metadata', title: 'Get Info', icon: 'info-circle',
      copy: 'One calm sheet with kind, size, dimensions, tags, and where the file lives. Rename and retag from the same place.',
      chips: ['Kind', 'Size', 'Tags']
    }
  });
  add({
    id: 'rename-selection', rail: 'files', group: 'Files', icon: 'text',
    title: 'Rename the selection', sub: 'Edit the name inline, in place',
    keywords: 'rename edit name label title',
    kbd: 'F2',
    run: () => {
      const selected = singleSelected();
      if (selected) startRename(selected);
      else toast('Select one file to rename', null, null, 'text');
    },
    detail: {
      eyebrow: 'Edit', title: 'Rename in place', icon: 'text',
      copy: 'The name becomes an inline field — no dialog, no extra chrome. Enter commits, Escape backs out.',
      chips: ['Inline', 'Enter commits', 'Undoable']
    }
  });
  add({
    id: 'tag-selection', rail: 'files', group: 'Files', icon: 'tag',
    title: 'Tag the selection', sub: 'Design · Amapiano · Archive',
    keywords: 'tag label design amapiano archive colour',
    run: () => {
      const selected = singleSelected();
      if (!selected) { toast('Select one file to tag', null, null, 'tag'); return; }
      const rect = $('#viewMount').getBoundingClientRect();
      openTagPicker(selected, rect.left + 120, rect.top + 120);
    },
    detail: {
      eyebrow: 'Organise', title: 'Tag the selection', icon: 'tag',
      copy: 'Tags are global — they collect files from every folder into one thread.',
      chips: ['Design', 'Amapiano', 'Archive']
    }
  });
  add({
    id: 'copy-to', rail: 'files', group: 'Files', icon: 'copy',
    title: 'Copy to…', sub: 'Pick a destination and watch it fly',
    keywords: 'copy duplicate destination transfer disk folder',
    run: () => {
      const selected = singleSelected();
      if (selected) beginTransfer('copy', selected);
      else toast('Select one file to copy', null, null, 'copy');
    },
    detail: {
      eyebrow: 'Transfer', title: 'Copy to…', icon: 'copy',
      copy: 'A three-column sheet: places on the left, destinations in the middle, and the file you are moving on the right.',
      chips: ['Local', 'External', 'Phone']
    }
  });
  add({
    id: 'move-to', rail: 'files', group: 'Files', icon: 'folder',
    title: 'Move to…', sub: 'Relocate without copy-paste',
    keywords: 'move relocate destination transfer disk folder',
    kbd: '⌘X',
    run: () => {
      const selected = singleSelected();
      if (selected) beginTransfer('move', selected);
      else toast('Select one file to move', null, null, 'folder');
    },
    detail: {
      eyebrow: 'Transfer', title: 'Move to…', icon: 'folder',
      copy: 'The file leaves its old home only once the hand-off lands. Undo puts it straight back.',
      chips: ['Atomic', 'Undoable']
    }
  });
  add({
    id: 'search-folder', rail: 'files', group: 'Files', icon: 'magnifer',
    title: `Search ${view.title}`, sub: 'Filter this folder inline',
    keywords: 'search find filter files folder',
    kbd: '/',
    run: () => openSearch(),
    detail: {
      eyebrow: 'Search', title: `Search ${view.title}`, icon: 'magnifer',
      copy: 'Jump into the appbar search. Results filter live as you type, without leaving the folder.',
      chips: [view.title, 'Live filter', 'Esc to clear'],
      stageMeta: ['Inline', 'No network']
    }
  });

  add({
    id: 'new-folder', rail: 'make', group: 'Create', icon: 'add-folder',
    title: 'New folder', sub: 'Create an untitled folder in Pictures',
    keywords: 'new folder create add untitled', kbd: '⇧⌘N', suggested: true,
    run: () => newFolder(),
    detail: {
      eyebrow: 'Create', title: 'New folder', icon: 'add-folder',
      copy: 'A fresh folder lands in Pictures, named and ready to rename. Nothing is moved or overwritten.',
      chips: ['Pictures', 'Instant', 'Undoable'],
      stageMeta: ['Untitled', 'Pictures']
    }
  });
  add({
    id: 'draft-cover', rail: 'make', group: 'Create', icon: 'stars-minimalistic',
    title: 'Draft an album cover', sub: 'Use the frosted glass texture',
    keywords: 'draft create artwork cover album amapiano ai',
    run: () => sendAI('Draft an album cover'),
    detail: {
      eyebrow: 'Create with AI', title: 'Draft an album cover', icon: 'stars-minimalistic', tone: 'var(--warn)',
      copy: 'Ask Smart to take the frosted glass texture and the log drum rhythm as the brief for three directions.',
      chips: ['1600 × 1600', '3 directions', 'On-device'],
      stageMeta: ['Frosted', 'Log drum']
    }
  });
  add({
    id: 'find-unused', rail: 'make', group: 'Ask Smart', icon: 'magnifer-zoom-in',
    title: 'Find unused images', sub: 'Scan the last 7 days of work',
    keywords: 'find unused images duplicates haven archive ai', suggested: true,
    run: () => sendAI("Find images I haven't used"),
    detail: {
      eyebrow: 'Smart action', title: 'Find unused images', icon: 'magnifer-zoom-in', tone: 'var(--warn)',
      copy: 'Scan the folder for images that have not appeared in a deck or a cover this week. Nothing moves without you.',
      chips: ['3 candidates', 'On-device', 'Read-only'],
      stageMeta: ['3 candidates', 'This week']
    }
  });
  add({
    id: 'organise', rail: 'make', group: 'Ask Smart', icon: 'gallery',
    title: 'Organise into albums', sub: 'Group by artwork, brand, and UI',
    keywords: 'organise organize albums tidy group ai artwork brand ui',
    run: () => sendAI('Organise these into albums'),
    detail: {
      eyebrow: 'Smart action', title: 'Organise into albums', icon: 'gallery',
      copy: 'Three albums, chosen from what is actually here: Album Artwork, Brand — Naledzi, and UI References.',
      chips: ['3 albums', 'Nothing moves', 'Review first'],
      stageMeta: ['Album artwork', 'Brand', 'UI']
    }
  });
  add({
    id: 'open-ask', rail: 'make', group: 'Ask Smart', icon: 'stars-minimalistic',
    title: 'Ask Smart about this folder', sub: `Open the panel with ${view.title} in context`,
    keywords: 'ask smart ai assistant folder context',
    run: () => { openAI(); requestAnimationFrame(() => $('#aiInput').focus()); },
    detail: {
      eyebrow: 'Ask Smart', title: `Ask about ${view.title}`, icon: 'stars-minimalistic',
      copy: 'The panel opens aware of this folder, its files, and its tags. Ask in plain language; nothing leaves the device.',
      chips: [view.title, 'On-device', 'Folder-aware'],
      stageMeta: ['Local', 'Aware']
    }
  });

  add({
    id: 'toggle-theme', rail: 'look', group: 'Look', icon: 'eye',
    title: 'Toggle appearance', sub: `Currently ${document.documentElement.dataset.theme === 'dark' ? 'dark' : 'light'}`,
    keywords: 'theme appearance dark light mode switch', suggested: true,
    run: () => toggleTheme(),
    detail: {
      eyebrow: 'Appearance', title: 'Light and dark', theme: true,
      copy: 'The same tonal ladder, tuned for a cool light room or a late session. The accent stays cobalt.',
      chips: ['Light', 'Dark', 'Instant']
    }
  });
  add({
    id: 'tokens', rail: 'look', group: 'Look', icon: 'widget',
    title: 'Open design tokens', sub: 'smart.toml · draft 0.1',
    keywords: 'tokens design system toml contract palette type elevation', kbd: 'T', suggested: true,
    run: () => openModal('tokens'),
    detail: {
      eyebrow: 'Design system', title: 'smart.toml', icon: 'widget',
      copy: 'The numbers behind this screen: palette, type, shape, space, elevation, motion. One contract, every host.',
      chips: ['v0.1', '5 elevations', 'WCAG'],
      stageMeta: ['smart.toml', 'Draft 0.1']
    }
  });
  add({
    id: 'view-grid', rail: 'look', group: 'Look', icon: 'widget',
    title: 'Grid view', sub: 'Large media, four columns', on: state.view === 'grid',
    keywords: 'grid view layout cards thumbnails',
    run: () => setView('grid'),
    detail: {
      eyebrow: 'Layout', title: 'Grid view', icon: 'widget',
      copy: 'The default surface: media first, generous cards, and quick actions on hover.',
      chips: ['4 columns', '16:10 media', 'Hover actions'],
      stageMeta: ['Media first', 'Default']
    }
  });
  add({
    id: 'view-list', rail: 'look', group: 'Look', icon: 'list-check',
    title: 'List view', sub: 'Dense rows with kinds and dates', on: state.view === 'list',
    keywords: 'list view rows table dense',
    run: () => setView('list'),
    detail: {
      eyebrow: 'Layout', title: 'List view', icon: 'list-check',
      copy: 'A tighter scan for comparing names, kinds, sizes, and dates across a folder.',
      chips: ['Rows', 'Sortable', 'Selection'],
      stageMeta: ['Dense', 'Comparable']
    }
  });
  add({
    id: 'toggle-sidebar', rail: 'look', group: 'Look', icon: 'sidebar-minimalistic',
    title: $('#win').classList.contains('is-collapsed') ? 'Show sidebar' : 'Hide sidebar',
    sub: 'Give the stage the full width',
    keywords: 'sidebar rail navigation show hide collapse',
    run: () => toggleSidebar(),
    detail: {
      eyebrow: 'Layout', title: 'Sidebar', icon: 'sidebar-minimalistic',
      copy: 'Collapse the rail and the traffic lights cross-fade into the appbar. The stage takes the full width.',
      chips: ['252 px', 'Cross-fade', '⌘\\'],
      stageMeta: ['Rail', 'Stage']
    }
  });
  add({
    id: 'grid-overlay', rail: 'look', group: 'Look', icon: 'magnifer-zoom-in',
    title: 'Grid overlay', sub: '8 px grid · 64 px major', on: gridOn(),
    keywords: 'grid overlay spacing alignment pixels',
    run: () => toggleGrid(),
    detail: {
      eyebrow: 'Craft', title: '8 px grid', icon: 'magnifer-zoom-in',
      copy: 'Reveal the baseline grid behind the surface. 8 px minor, 64 px major, 4 px baseline.',
      chips: ['8 px', '64 px major', '4 px baseline'],
      stageMeta: ['8 px', 'Major 64']
    }
  });
  add({
    id: 'copy-tokens', rail: 'look', group: 'Look', icon: 'copy',
    title: 'Copy design tokens', sub: 'Put smart.toml on the clipboard',
    keywords: 'copy tokens toml clipboard export',
    run: () => copyText(TOML_SOURCE),
    detail: {
      eyebrow: 'Design system', title: 'Copy smart.toml', icon: 'copy',
      copy: 'Copy the full contract exactly as it ships beside the prototype, comments included.',
      chips: ['Clipboard', '5.4 KB', 'Comments included'],
      stageMeta: ['smart.toml', 'Exact']
    }
  });
  add({
    id: 'wallpaper', rail: 'look', group: 'Look', icon: 'gallery-wide',
    title: 'Change the wallpaper', sub: 'Four surfaces behind the window',
    keywords: 'wallpaper desktop background surface picture theme',
    run: () => openWallpaper(),
    detail: {
      eyebrow: 'Surface', title: 'Desktop wallpaper', icon: 'gallery-wide',
      copy: 'The palette stays anchored to cobalt — only the surface behind the glass changes.',
      chips: ['4 surfaces', 'Live', 'Local']
    }
  });
  add({
    id: 'shortcuts', rail: 'look', group: 'Look', icon: 'list-check',
    title: 'Keyboard shortcuts', sub: 'Every binding in one sheet',
    keywords: 'shortcuts keyboard keys help reference bindings',
    kbd: '?',
    run: () => openShortcuts(),
    detail: {
      eyebrow: 'Reference', title: 'Keyboard shortcuts', icon: 'list-check',
      copy: 'The same language the command palette speaks. Nothing here needs a mouse.',
      chips: [String(SHORTCUTS.length) + ' bindings', '⌘K first']
    }
  });
  add({
    id: 'about', rail: 'look', group: 'Look', icon: 'stars-minimalistic',
    title: 'About Smart Files', sub: '0.1 · built on smart-protocol',
    keywords: 'about version build info smart files workspace',
    run: () => openAbout(),
    detail: {
      eyebrow: 'About', title: 'Smart Files 0.1', icon: 'stars-minimalistic',
      copy: 'A local-first file surface for the Smart suite — built on smart-protocol, one smart.toml, no network.',
      chips: ['0.1', 'Local-first', 'smart-protocol']
    }
  });
  add({
    id: 'new-window', rail: 'make', group: 'Create', icon: 'maximize',
    title: 'New window', sub: 'Open a second Smart Files window',
    keywords: 'new window open duplicate instance',
    run: () => newSmartWindow(),
    detail: {
      eyebrow: 'Window', title: 'New window', icon: 'maximize',
      copy: 'A second window opens on the same workspace — same tokens, same surface, its own navigation.',
      chips: ['Same workspace', 'Same tokens']
    }
  });

  return cmds;
}

function paletteMatches(command, query) {
  const haystack = `${command.title} ${command.sub} ${command.keywords || ''} ${command.group} ${command.rail}`.toLowerCase();
  return query.split(/\s+/).filter(Boolean).every(part => haystack.includes(part));
}

function paletteGroups() {
  const all = paletteCommands();
  const query = state.paletteQuery.trim().toLowerCase();
  let pool = state.paletteRail === 'all' ? all : all.filter(command => command.rail === state.paletteRail);
  if (query) pool = pool.filter(command => paletteMatches(command, query));

  if (state.paletteRail === 'all' && !query) {
    const suggested = all.filter(command => command.suggested);
    const groups = [{ label: 'Suggested', items: suggested }];
    const rest = all.filter(command => !command.suggested);
    PALETTE_GROUP_ORDER.forEach(label => {
      const items = rest.filter(command => command.group === label);
      if (items.length) groups.push({ label, items });
    });
    return groups;
  }

  const groups = [];
  PALETTE_GROUP_ORDER.forEach(label => {
    const items = pool.filter(command => command.group === label);
    if (items.length) groups.push({ label, items });
  });
  return groups;
}

function paletteRowHTML(command, index) {
  const selected = index === state.paletteIndex;
  const tail = command.on
    ? '<span class="cmd-state">On</span>'
    : command.kbd
      ? `<kbd class="cmd-kbd">${esc(command.kbd)}</kbd>`
      : '<span class="ic" data-ic="arrow-right-up"></span>';
  return `<button class="cmd-row${selected ? ' sel' : ''}" id="cmdrow-${index}" data-cmd-index="${index}" role="option" aria-selected="${selected}">
    <span class="cmd-row-ic"><span class="ic" data-ic="${command.icon}"></span></span>
    <span class="cmd-row-copy"><b>${esc(command.title)}</b><small>${esc(command.sub)}</small></span>
    <span class="cmd-tail">${tail}</span>
  </button>`;
}

function paletteEmptyHTML() {
  const query = state.paletteQuery.trim();
  return `<div class="cmd-empty">
    <span class="cmd-empty-orb"><span class="ic" data-ic="magnifer"></span></span>
    <h4>No command for “${esc(query)}”</h4>
    <p>Try a file name, a place, or ask Smart to find it for you.</p>
    <div class="cmd-empty-actions">
      <button data-cmd-empty="search">Search this folder</button>
      <button class="primary" data-cmd-empty="ask">Ask Smart</button>
    </div>
  </div>`;
}

function paletteStageHTML(command) {
  const detail = command.detail || {};
  if (detail.theme) {
    return `<div class="cmd-theme">
      <div class="cmd-theme-pane light"><i></i><i></i><i></i><i></i><span class="cmd-theme-label">Light</span></div>
      <div class="cmd-theme-pane dark"><i></i><i></i><i></i><i></i><span class="cmd-theme-label">Dark</span></div>
    </div>`;
  }
  if (detail.thumbs && detail.thumbs.length >= 3) {
    return `<div class="cmd-mosaic">${detail.thumbs.slice(0, 3).map(thumb =>
      `<img src="assets/thumbs/${esc(thumb)}.jpg" alt="" draggable="false">`).join('')}</div>`;
  }
  if (detail.thumb) {
    return `<div class="cmd-shot">
      <img src="assets/thumbs/${esc(detail.thumb)}.jpg" alt="" draggable="false">
      ${detail.tag ? `<span class="cmd-shot-tag">${esc(detail.tag)}</span>` : ''}
      ${detail.shotName ? `<span class="cmd-shot-name">${esc(detail.shotName)}</span>` : ''}
    </div>`;
  }
  const meta = (detail.stageMeta || []).slice(0, 2).map((item, index) =>
    `<span>${index === 0 ? '<span class="ic" data-ic="check-circle"></span>' : ''}${esc(item)}</span>`).join('');
  return `<div class="cmd-stage" style="--tone:${detail.tone || 'var(--accent)'}">
    <span class="cmd-stage-orb"><span class="ic" data-ic="${detail.icon || command.icon}"></span></span>
    ${meta ? `<div class="cmd-stage-meta">${meta}</div>` : ''}
  </div>`;
}

function renderPaletteDetail() {
  const detail = $('#cmdDetail');
  const command = state.paletteItems[state.paletteIndex];
  const total = state.paletteItems.length;
  if (!command) {
    detail.innerHTML = `<div class="cmd-detail-head"><span>Context</span><span class="cmd-dcount">00 / 00</span></div>
      <div class="cmd-detail-empty">
        <span class="cmd-empty-orb"><span class="ic" data-ic="stars-minimalistic"></span></span>
        <h4>Ask Smart instead</h4>
        <p>No command matches this query. Smart can search inside the folder and explain what it finds.</p>
      </div>`;
    hydrateIcons(detail);
    return;
  }

  const copy = command.detail || {};
  const count = `${String(state.paletteIndex + 1).padStart(2, '0')} / ${String(total).padStart(2, '0')}`;
  const chips = (copy.chips || []).map(chip => `<span class="cmd-chip">${esc(chip)}</span>`).join('');
  detail.innerHTML = `<div class="cmd-detail-head"><span>${esc(copy.eyebrow || 'Selected')}</span><span class="cmd-dcount">${count}</span></div>
    ${paletteStageHTML(command)}
    <div class="cmd-dtitle">${esc(copy.title || command.title)}</div>
    <p class="cmd-dcopy">${esc(copy.copy || command.sub)}</p>
    ${chips ? `<div class="cmd-chips">${chips}</div>` : ''}
    <div class="cmd-run-wrap">
      <button class="cmd-run" id="cmdRun"><span>${esc(copy.cta || 'Run command')}</span><kbd>↵</kbd></button>
      <div class="cmd-safe"><i></i>Runs locally · nothing leaves this device</div>
    </div>`;
  hydrateIcons(detail);
}

function renderPalette() {
  const groups = paletteGroups();
  state.paletteItems = groups.flatMap(group => group.items);
  if (!state.paletteItems.length) state.paletteIndex = 0;
  else state.paletteIndex = Math.max(0, Math.min(state.paletteIndex, state.paletteItems.length - 1));

  let index = 0;
  const list = $('#cmdList');
  list.innerHTML = groups.length
    ? groups.map(group => `<div class="cmd-group">
        <div class="cmd-group-label">${esc(group.label)}<span>${String(group.items.length).padStart(2, '0')}</span></div>
        ${group.items.map(command => paletteRowHTML(command, index++)).join('')}
      </div>`).join('')
    : paletteEmptyHTML();

  $$('#cmdRails .cmd-rail-btn').forEach(button => {
    const active = button.dataset.rail === state.paletteRail;
    button.classList.toggle('active', active);
    button.setAttribute('aria-pressed', String(active));
  });
  $('#cmdTitle').textContent = state.paletteRail === 'all'
    ? 'Command surface'
    : `${PALETTE_RAIL_LABEL[state.paletteRail]} commands`;
  const input = $('#cmdInput');
  if (state.paletteItems.length) input.setAttribute('aria-activedescendant', `cmdrow-${state.paletteIndex}`);
  else input.removeAttribute('aria-activedescendant');
  hydrateIcons(list);
  renderPaletteDetail();
}

function selectPaletteIndex(index, scroll = false) {
  if (!state.paletteItems.length) return;
  const total = state.paletteItems.length;
  state.paletteIndex = ((index % total) + total) % total;
  $$('#cmdList .cmd-row').forEach((row, rowIndex) => {
    const selected = rowIndex === state.paletteIndex;
    row.classList.toggle('sel', selected);
    row.setAttribute('aria-selected', String(selected));
  });
  $('#cmdInput').setAttribute('aria-activedescendant', `cmdrow-${state.paletteIndex}`);
  renderPaletteDetail();
  if (scroll) {
    const row = $(`#cmdrow-${state.paletteIndex}`);
    if (row) row.scrollIntoView({ block: 'nearest' });
  }
}

function setPaletteRail(rail) {
  if (!PALETTE_RAILS.includes(rail)) return;
  state.paletteRail = rail;
  state.paletteIndex = 0;
  renderPalette();
  const input = $('#cmdInput');
  input.focus();
}

function cyclePaletteRail(direction) {
  const current = PALETTE_RAILS.indexOf(state.paletteRail);
  setPaletteRail(PALETTE_RAILS[(current + direction + PALETTE_RAILS.length) % PALETTE_RAILS.length]);
}

function runPaletteCommand(command) {
  if (!command) return;
  closePalette(false);
  command.run();
}

function runSelectedPaletteCommand() {
  runPaletteCommand(state.paletteItems[state.paletteIndex]);
}

function askFromPalette() {
  const query = state.paletteQuery.trim();
  closePalette(false);
  openAI();
  if (query) {
    const input = $('#aiInput');
    input.value = query;
    input.dispatchEvent(new Event('input', { bubbles: true }));
  }
  requestAnimationFrame(() => {
    const input = $('#aiInput');
    input.focus();
    input.setSelectionRange(input.value.length, input.value.length);
  });
}

function searchFromPalette() {
  const query = state.paletteQuery.trim();
  closePalette(false);
  if (query) setQuery(query);
  openSearch();
}

function openPalette(rail = 'all') {
  if (state.paletteOpen) {
    closePalette();
    return;
  }
  closeMenus();
  closeModals();
  closeAI();
  state.paletteOpen = true;
  state.paletteRail = PALETTE_RAILS.includes(rail) ? rail : 'all';
  state.paletteQuery = '';
  state.paletteIndex = 0;
  state.paletteLastFocus = document.activeElement;
  const layer = $('#paletteLayer');
  layer.classList.add('show');
  layer.setAttribute('aria-hidden', 'false');
  $('#cmdBtn').setAttribute('aria-expanded', 'true');
  $('#cmdInput').value = '';
  renderPalette();
  requestAnimationFrame(() => {
    const input = $('#cmdInput');
    input.focus();
    input.setSelectionRange(input.value.length, input.value.length);
  });
}

function closePalette(restoreFocus = true) {
  if (!state.paletteOpen) return;
  state.paletteOpen = false;
  $('#paletteLayer').classList.remove('show');
  $('#paletteLayer').setAttribute('aria-hidden', 'true');
  $('#cmdBtn').setAttribute('aria-expanded', 'false');
  state.paletteQuery = '';
  state.paletteItems = [];
  state.paletteIndex = 0;
  const target = state.paletteLastFocus;
  state.paletteLastFocus = null;
  if (restoreFocus && target && document.contains(target)) {
    try { target.focus(); } catch (error) { /* focus guard */ }
  }
}

/* ============================================================
   preview · tokens · AI content
   ============================================================ */

function renderTokens() {
  const code = $('#tokenCode');
  code.innerHTML = highlightTOML(TOML_SOURCE);
}

function highlightTOML(source) {
  return source.split('\n').map(line => {
    if (/^\s*#/.test(line)) return `<span class="tok-com">${esc(line)}</span>`;

    const section = line.match(/^(\s*)(\[[^\]]+\])(\s*#.*)?$/);
    if (section) {
      const comment = section[3] ? `<span class="tok-com">${esc(section[3])}</span>` : '';
      return `${section[1]}<span class="tok-sec">${esc(section[2])}</span>${comment}`;
    }

    if (/^\s*"/.test(line)) return `<span class="tok-str">${esc(line)}</span>`;

    const pair = line.match(/^(\s*)([A-Za-z0-9_.-]+)(\s*=\s*)(.*)$/);
    if (pair) {
      const indent = pair[1];
      const key = pair[2];
      let value = pair[4];
      let comment = '';
      const hash = value.indexOf(' #');
      if (hash >= 0) {
        comment = value.slice(hash);
        value = value.slice(0, hash);
      }
      let valueHtml;
      if (/^\s*"/.test(value)) valueHtml = `<span class="tok-str">${esc(value)}</span>`;
      else if (/^\s*(true|false)\b/.test(value)) valueHtml = `<span class="tok-bool">${esc(value)}</span>`;
      else if (/^\s*-?\d/.test(value)) valueHtml = `<span class="tok-num">${esc(value)}</span>`;
      else valueHtml = `<span class="tok-str">${esc(value)}</span>`;
      return `${indent}<span class="tok-key">${esc(key)}</span><span class="tok-punc"> = </span>${valueHtml}${comment ? `<span class="tok-com">${esc(comment)}</span>` : ''}`;
    }

    return esc(line);
  }).join('\n');
}

async function copyText(text) {
  try {
    await navigator.clipboard.writeText(text);
    toast('smart.toml copied');
  } catch (error) {
    const area = document.createElement('textarea');
    area.value = text;
    area.style.position = 'fixed';
    area.style.opacity = '0';
    document.body.appendChild(area);
    area.select();
    try { document.execCommand('copy'); toast('smart.toml copied'); }
    catch (err) { toast('Copy is unavailable here'); }
    area.remove();
  }
}

function appendAIMessage(text, who = 'user') {
  const body = $('#aiBody');
  const el = document.createElement('div');
  el.className = who === 'user' ? 'ai-msg ai-user' : 'ai-msg';

  if (who === 'user') {
    el.innerHTML = `<div class="ai-user-bubble">${esc(text)}</div>`;
  } else {
    el.innerHTML = `<span class="ai-avatar"><span class="ic" data-ic="stars-minimalistic"></span></span><div class="ai-copy">${text}</div>`;
  }

  body.appendChild(el);
  hydrateIcons(el);
  body.scrollTop = body.scrollHeight;
}

function aiReply(prompt) {
  const text = prompt.toLowerCase();
  if (/(organis|organiz|album|tidy)/.test(text)) {
    return 'I would group this folder into <b>three albums</b>: <b>Album Artwork</b> (2), <b>Brand — Naledzi</b> (3), and <b>UI References</b> (3). Nothing moves until you say so.';
  }
  if (/(unused|haven|duplicate|find)/.test(text)) {
    return 'Three images have not appeared in a deck this week: <b>Green Fold.png</b>, <b>Cycling in Soweto.png</b>, and <b>Blossom Study.jpg</b>. I can file them under Archive.';
  }
  if (/(cover|artwork|draft|mockup)/.test(text)) {
    return 'I would take the <b>frosted glass</b> texture and the <b>log drum</b> rhythm as the brief. Three 1600 × 1600 cover directions are ready to review.';
  }
  return 'I can work with this folder. Ask me to organise it, find unused images, or draft artwork from what is here.';
}

function sendAI(message) {
  const input = $('#aiInput');
  const text = (message !== undefined ? message : input.value).trim();
  if (!text) return;
  openAI();
  appendAIMessage(text, 'user');
  input.value = '';
  input.style.height = '38px';

  const body = $('#aiBody');
  const typing = document.createElement('div');
  typing.className = 'ai-msg';
  typing.innerHTML = `<span class="ai-avatar"><span class="ic" data-ic="stars-minimalistic"></span></span><div class="ai-copy"><span class="ai-typing"><i></i><i></i><i></i></span></div>`;
  body.appendChild(typing);
  hydrateIcons(typing);
  body.scrollTop = body.scrollHeight;

  setTimeout(() => {
    typing.remove();
    appendAIMessage(aiReply(text), 'ai');
  }, 850);
}

/* ============================================================
   menus
   ============================================================ */

function buildMenu(items, key = null) {
  const menu = document.createElement('div');
  menu.className = 'menu';
  menu.setAttribute('role', 'menu');
  menu._items = items;
  menu._key = key;

  menu.innerHTML = items.map((item, index) => {
    if (item.sep) return '<div class="msep" role="separator"></div>';
    if (item.header) return `<div class="mlabel">${esc(item.header)}</div>`;
    const classes = ['mi'];
    if (item.danger) classes.push('danger');
    if (item.sub) classes.push('sub');
    if (item.on) classes.push('on');
    let right = '';
    if (item.sub) right = '<span class="chev" aria-hidden="true"></span>';
    else if (item.on) right = '<span class="ic check" data-ic="check-circle"></span>';
    else if (item.kbd) right = `<span class="mk">${esc(item.kbd)}</span>`;
    return `<button class="${classes.join(' ')}" data-mi="${index}" role="menuitem" ${item.disabled ? 'disabled' : ''}>
      ${item.icon ? `<span class="ic" data-ic="${item.icon}"></span>` : ''}
      <span class="ml">${esc(item.label)}</span>${right}
    </button>`;
  }).join('');

  return menu;
}

function clampMenu(menu, x, y) {
  const rect = $('#win').getBoundingClientRect();
  const width = menu.offsetWidth;
  const height = menu.offsetHeight;
  let left = x - rect.left;
  let top = y - rect.top;
  left = Math.max(10, Math.min(left, rect.width - width - 10));
  top = Math.max(10, Math.min(top, rect.height - height - 10));
  menu.style.left = `${left}px`;
  menu.style.top = `${top}px`;
}

function showMenu(items, x, y, key = null) {
  closeMenus();
  state.lastMenuPos = { x, y };
  const menu = buildMenu(items, key);
  menu.style.left = '-9999px';
  menu.style.top = '-9999px';
  $('#menuLayer').appendChild(menu);
  hydrateIcons(menu);
  clampMenu(menu, x, y);
  state.menuOpen = true;
  return menu;
}

function closeMenus() {
  const layer = $('#menuLayer');
  if (layer) layer.innerHTML = '';
  state.menuOpen = false;
  state.submenu = null;
}

function closeSubmenus() {
  $$('#menuLayer .menu.sub').forEach(el => el.remove());
  $$('#menuLayer .mi.open').forEach(el => el.classList.remove('open'));
  state.submenu = null;
}

function openSubmenu(itemEl, subItems) {
  closeSubmenus();
  const parent = itemEl.closest('.menu');
  const submenu = buildMenu(subItems, parent ? parent._key : null);
  submenu.classList.add('sub');
  submenu.style.left = '-9999px';
  submenu.style.top = '-9999px';
  $('#menuLayer').appendChild(submenu);
  hydrateIcons(submenu);

  const win = $('#win').getBoundingClientRect();
  const rect = itemEl.getBoundingClientRect();
  let left = rect.right - win.left + 3;
  if (left + submenu.offsetWidth > win.width - 8) {
    left = rect.left - win.left - submenu.offsetWidth - 3;
  }
  let top = rect.top - win.top - 6;
  if (top + submenu.offsetHeight > win.height - 8) {
    top = win.height - submenu.offsetHeight - 8;
  }
  if (top < 8) top = 8;

  submenu.style.left = `${left}px`;
  submenu.style.top = `${top}px`;
  itemEl.classList.add('open');
  state.submenu = submenu;
}

function fileMenu(item) {
  const isFolder = !!item.items;
  const isImage = !!item.thumb;
  const kind = previewKind(item);
  return [
    { icon: isFolder ? 'folder' : isImage ? 'gallery' : 'arrow-right-up', label: 'Open', kbd: 'Return', action: 'open' },
    {
      icon: 'widget', label: 'Open With', sub: [
        ...(kind === 'image' ? [{ icon: 'eye', label: 'Preview', action: 'preview' }] : []),
        ...(kind === 'audio' ? [{ icon: 'music-notes', label: 'Smart DAW', action: 'openwith', app: 'Smart DAW' }] : []),
        ...(kind === 'doc' ? [{ icon: 'document-text', label: 'Smart Reader', action: 'openwith', app: 'Smart Reader' }] : []),
        ...(kind === 'app' ? [{ icon: 'widget', label: 'Launch smart-daw', action: 'openwith', app: 'smart-daw' }] : []),
        { icon: 'gallery-wide', label: 'Smart Image Viewer', action: 'openwith', app: 'Smart Image Viewer' },
        { icon: 'music-notes', label: 'Smart DAW', action: 'openwith', app: 'Smart DAW' },
        { icon: 'global', label: 'Default app', action: 'openwith', app: 'the default app' },
        { sep: true },
        { icon: 'menu-dots', label: 'Other…', action: 'openwith', app: 'another app' }
      ]
    },
    { sep: true },
    { icon: 'eye', label: 'Quick Look', kbd: 'Space', action: 'preview', disabled: !kind },
    { icon: 'info-circle', label: 'Get Info', kbd: '⌘I', action: 'info' },
    { icon: 'share', label: 'Share…', action: 'share' },
    { sep: true },
    { icon: 'copy', label: 'Copy', kbd: '⌘C', action: 'copy' },
    { icon: 'folder', label: 'Copy to…', action: 'copyto' },
    { icon: 'arrow-right-up', label: 'Move to…', kbd: '⌘X', action: 'move' },
    { icon: 'text', label: 'Rename', kbd: 'F2', action: 'rename' },
    { icon: 'tag', label: 'Add Tag', action: 'tag' },
    { sep: true },
    { icon: 'users-group-rounded', label: 'Send to Phone', action: 'sendphone' },
    { icon: 'trash-bin-minimalistic', label: 'Move to Trash', kbd: '⌘⌫', action: 'trash', danger: true }
  ];
}

function sortMenuItems() {
  return [
    { header: 'Sort by' },
    { icon: 'text', label: 'Name', on: state.sort === 'name', action: 'sort', value: 'name' },
    { icon: 'file', label: 'Kind', on: state.sort === 'kind', action: 'sort', value: 'kind' },
    { icon: 'clock-circle', label: 'Date Modified', on: state.sort === 'date', action: 'sort', value: 'date' },
    { icon: 'archive', label: 'Size', on: state.sort === 'size', action: 'sort', value: 'size' }
  ];
}

function emptyMenuItems() {
  return [
    { icon: 'add-folder', label: 'New Folder', kbd: '⇧⌘N', action: 'newfolder' },
    { icon: 'copy', label: 'Paste', kbd: '⌘V', action: 'paste', disabled: !state.clipboardList.length && !state.clipboard },
    { sep: true },
    { icon: 'sort-from-top-to-bottom', label: 'Sort By', sub: sortMenuItems().slice(1) },
    { icon: 'widget', label: 'Show View Options', action: 'viewopts' },
    { icon: 'magnifer-zoom-in', label: 'Grid Overlay', on: gridOn(), action: 'gridoverlay' },
    { sep: true },
    { icon: 'gallery-wide', label: 'Change Wallpaper…', action: 'wallpaper' },
    { icon: 'ssd-square', label: 'Devices & Locations…', action: 'devices' },
    { icon: 'info-circle', label: 'Get Info', kbd: '⌘I', action: 'locationinfo' }
  ];
}

function moreMenuItems() {
  const collapsed = $('#win').classList.contains('is-collapsed');
  return [
    { icon: 'add-folder', label: 'New Folder', kbd: '⇧⌘N', action: 'newfolder' },
    { icon: 'maximize', label: 'New Window', kbd: '⌘N', action: 'newwindow' },
    { sep: true },
    { icon: 'sidebar-minimalistic', label: collapsed ? 'Show Sidebar' : 'Hide Sidebar', kbd: '⌘\\', action: 'sidebar' },
    { icon: 'magnifer', label: 'Command palette…', kbd: '⌘K', action: 'palette' },
    { icon: 'widget', label: 'Design Tokens…', kbd: 'T', action: 'tokens' },
    { icon: 'stars-minimalistic', label: 'Toggle Appearance', kbd: 'D', action: 'theme' },
    { icon: 'magnifer-zoom-in', label: 'Grid Overlay', on: gridOn(), action: 'gridoverlay' },
    { sep: true },
    { icon: 'ssd-square', label: 'Devices & Locations…', action: 'devices' },
    { icon: 'users-group-rounded', label: 'Phone Continuity…', action: 'phone' },
    { icon: 'list-check', label: 'Keyboard Shortcuts…', kbd: '?', action: 'shortcuts' },
    { icon: 'gallery-wide', label: 'Change Wallpaper…', action: 'wallpaper' },
    { sep: true },
    { icon: 'info-circle', label: 'About Smart Files', action: 'about' }
  ];
}

function workspaceMenuItems() {
  return [
    { header: 'Workspaces' },
    { icon: 'stars-minimalistic', label: 'Naledzi — Personal', on: true, action: 'workspace', value: 'Naledzi', subvalue: 'Personal workspace' },
    { icon: 'users-group-rounded', label: 'Smart Studio', action: 'workspace', value: 'Smart Studio', subvalue: 'Shared workspace' },
    { icon: 'global', label: 'Guest session', action: 'workspace', value: 'Guest', subvalue: 'Temporary session' },
    { sep: true },
    { icon: 'widget', label: 'Workspace details…', action: 'locationinfo' }
  ];
}
/* ============================================================
   real surfaces · every option does something
   ============================================================ */

const WALLPAPERS = [
  { id: 'frosted', name: 'Frosted 4K', sub: 'Default', base: '#9aa7bb', image: 'url("assets/wallpaper.jpg")' },
  { id: 'dusk', name: 'Cobalt Dusk', sub: 'Cool', base: '#0b1220', image: 'linear-gradient(150deg,#3c5686 0%,#1b2740 48%,#0a1120 100%)' },
  { id: 'sand', name: 'Sandstone', sub: 'Warm', base: '#8e867b', image: 'linear-gradient(150deg,#e8e0d2 0%,#c8bfb1 48%,#8e867b 100%)' },
  { id: 'mint', name: 'Mint Glass', sub: 'Fresh', base: '#6b9185', image: 'linear-gradient(150deg,#dcefe6 0%,#a9c9bd 48%,#6b9185 100%)' }
];

const SHORTCUTS = [
  ['Command palette', '⌘K'],
  ['Search this folder', '/'],
  ['Find', '⌘F'],
  ['Toggle sidebar', '⌘\\'],
  ['Back · Forward', '⌘[ · ⌘]'],
  ['Select all', '⌘A'],
  ['Copy · Cut · Paste', '⌘C · ⌘X · ⌘V'],
  ['Get Info', '⌘I'],
  ['Move to Trash', '⌘⌫'],
  ['New folder', '⇧⌘N'],
  ['Quick Look', 'Space'],
  ['Open selection', '↵'],
  ['Design tokens', 'T'],
  ['Grid overlay', 'G'],
  ['Toggle appearance', 'D'],
  ['Shortcut reference', '?'],
  ['Move in palette', '↑ ↓'],
  ['Switch palette rail', 'Tab'],
  ['Close any surface', 'Esc']
];

let audioTimer = null;
let appTimer = null;

function fmtTime(seconds) {
  const total = Math.max(0, Math.round(seconds));
  return `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`;
}

/* ---------- rename ---------- */

function renameItem(key, name) {
  const found = findItem(key);
  if (!found || !name) return;
  found.item.name = name;
}

function copyName(name) {
  const dot = name.lastIndexOf('.');
  if (dot <= 0) return `${name} copy`;
  return `${name.slice(0, dot)} copy${name.slice(dot)}`;
}

function startRename(key) {
  const found = findItem(key);
  if (!found) return;
  const host = $(`[data-key="${key}"]`);
  if (!host) return;
  const target = host.querySelector('.cname span') || host.querySelector('.lname') || host.querySelector('.fname');
  if (!target) return;

  const input = document.createElement('input');
  input.className = 'rename-input';
  input.value = found.item.name;
  input.setAttribute('aria-label', `Rename ${found.item.name}`);
  const dot = found.item.name.lastIndexOf('.');
  target.replaceWith(input);
  input.focus();
  if (dot > 0) input.setSelectionRange(0, dot);
  else input.select();

  let settled = false;
  const finish = commit => {
    if (settled) return;
    settled = true;
    const value = input.value.trim();
    if (commit && value && value !== found.item.name) {
      renameItem(key, value);
      toast(`Renamed to “${value}”`, 'Undo', () => {
        renameItem(key, found.item.name);
        render();
      });
    }
    render();
  };
  input.addEventListener('keydown', event => {
    if (event.key === 'Enter') { event.preventDefault(); finish(true); }
    else if (event.key === 'Escape') { event.preventDefault(); finish(false); }
  });
  input.addEventListener('blur', () => finish(true));
}

/* ---------- copy · paste · tag ---------- */

function copyItem(key) {
  const found = findItem(key);
  if (!found) return;
  state.clipboard = key;
  state.clipboardList = [key];
  const link = `smart://files/${encodeURIComponent(found.item.name)}`;
  if (navigator.clipboard && navigator.clipboard.writeText) {
    navigator.clipboard.writeText(link).catch(() => {});
  }
  toast(`Copied “${found.item.name}”`, 'Paste', () => pasteItem(), 'copy');
}

function copySelection() {
  const keys = [...state.selection];
  if (!keys.length) {
    toast('Select something to copy', null, null, 'copy');
    return;
  }
  state.clipboardList = keys;
  state.clipboard = keys[0];
  const first = findItem(keys[0]);
  const label = keys.length === 1 && first
    ? `“${first.item.name}”`
    : `${keys.length} items`;
  toast(`Copied ${label}`, 'Paste', () => pasteItem(), 'copy');
}

function removeItemByKey(key) {
  const index = FILES.findIndex(file => file.key === key);
  if (index >= 0) { FILES.splice(index, 1); return; }
  const folderIndex = FOLDERS.findIndex(folder => folder.key === key);
  if (folderIndex >= 0) { FOLDERS.splice(folderIndex, 1); return; }
  for (const folder of FOLDERS) {
    const inner = folder.items.findIndex(item => item.key === key);
    if (inner >= 0) { folder.items.splice(inner, 1); return; }
  }
  for (const device of state.devices) {
    for (const list of [device.files, device.backups, device.pack]) {
      if (!list) continue;
      const at = list.findIndex(item => item.key === key);
      if (at >= 0) { list.splice(at, 1); return; }
    }
  }
  for (const list of [state.phoneFiles, state.documentsFiles, state.downloadsFiles]) {
    const at = list.findIndex(item => item.key === key);
    if (at >= 0) { list.splice(at, 1); return; }
  }
}

function pasteInto(container, source, where, mode = 'copy') {
  const stamp = Date.now();
  const copy = { ...source, key: `${source.key}-${mode}-${stamp}`, name: copyName(source.name), date: 'Just now' };
  if (source.items) {
    copy.items = source.items.map(item => ({ ...item, key: `${item.key}-${mode}-${stamp}` }));
  }
  container.list.unshift(copy);
  return copy;
}

function pasteItem() {
  const keys = state.clipboardList.length ? state.clipboardList : (state.clipboard ? [state.clipboard] : []);
  if (!keys.length) {
    toast('Nothing to paste', null, null, 'copy');
    return;
  }

  let where = 'Pictures';
  let container = null;
  if (state.route.type === 'folder') {
    const folder = FOLDERS.find(item => item.id === state.route.id);
    if (folder) {
      container = { list: folder.items, label: folder.name };
      where = folder.name;
    }
  } else if (state.route.type === 'device') {
    const device = state.devices.find(item => item.id === state.route.id);
    if (device && device.mounted) {
      if (!device.files) device.files = [];
      container = { list: device.files, label: device.name };
      where = device.name;
    }
  }
  if (!container) container = { list: FILES, label: 'Pictures' };

  const made = [];
  keys.forEach(key => {
    const found = findItem(key);
    if (!found) return;
    if (found.item.items) {
      FOLDERS.unshift(pasteInto(FOLDERS, found.item, where));
    } else {
      made.push(pasteInto(container, found.item, where));
    }
  });

  render();
  const label = made.length === 1 ? `“${made[0].name}”` : `${made.length} items`;
  toast(`Pasted ${label} in ${where}`, 'Undo', () => {
    made.forEach(item => removeItemByKey(item.key));
    render();
  }, 'copy');
}

function setItemTag(key, tag) {
  const found = findItem(key);
  if (!found) return;
  if (tag) found.item.tag = tag;
  else delete found.item.tag;
  render();
  toast(tag ? `Tagged “${found.item.name}” as ${TAG[tag].label}` : `Removed the tag from “${found.item.name}”`, null, null, 'tag');
}

function openTagPicker(key, x, y) {
  const found = findItem(key);
  if (!found) return;
  const items = Object.keys(TAG).map(id => ({
    icon: 'tag',
    label: TAG[id].label,
    on: found.item.tag === id,
    action: 'settag',
    value: id
  }));
  items.push({ sep: true });
  items.push({
    icon: 'close-circle',
    label: 'No tag',
    on: !found.item.tag,
    action: 'settag',
    value: ''
  });
  showMenu(items, x, y, key);
}

function openViewOptions(x, y) {
  showMenu([
    { icon: 'text', label: 'Show file extensions', on: state.opts.ext, action: 'opt-ext' },
    { icon: 'clock-circle', label: 'Show dates', on: state.opts.dates, action: 'opt-dates' },
    { icon: 'widget', label: 'Compact grid', on: state.opts.density === 'compact', action: 'opt-density' }
  ], x, y);
}

function toggleOption(name) {
  if (name === 'ext') state.opts.ext = !state.opts.ext;
  if (name === 'dates') state.opts.dates = !state.opts.dates;
  if (name === 'density') state.opts.density = state.opts.density === 'compact' ? 'comfortable' : 'compact';
  render();
  const labels = {
    ext: state.opts.ext ? 'Extensions shown' : 'Extensions hidden',
    dates: state.opts.dates ? 'Dates shown' : 'Dates hidden',
    density: state.opts.density === 'compact' ? 'Compact grid on' : 'Comfortable grid on'
  };
  toast(labels[name], null, null, 'widget');
}

/* ---------- get info ---------- */

function renderInfo(item, locationName, extra) {
  const isFolder = !!item.items;
  const size = isFolder ? fmt(item.bytes || folderBytes(item)) : (item.size || fmt(item.bytes || 0));

  $('#infoName').textContent = item.name;
  $('#infoSub').textContent = `${isFolder ? 'Folder' : (item.kindLabel || 'File')} · ${size}`;

  const hero = $('#infoHero');
  if (item.thumb) hero.innerHTML = `<img src="assets/thumbs/${item.thumb}.jpg" alt="${esc(item.name)}">`;
  else hero.innerHTML = `<span class="ic" data-ic="${item.icon || (isFolder ? 'folder' : 'file')}"></span>`;

  const cells = [
    ['Kind', isFolder ? 'Folder' : (item.kindLabel || 'File')],
    ['Size', size],
    isFolder ? ['Contains', `${item.items.length} items`] : ['Dimensions', item.dims || '—'],
    ['Modified', item.date || 'Today'],
    ['Location', locationName || 'Pictures'],
    ['Type', isFolder ? 'Folder' : (item.ext || 'FILE')]
  ];
  $('#infoGrid').innerHTML = cells.map(([label, value]) =>
    `<div class="info-cell"><span>${esc(label)}</span><b>${esc(value)}</b></div>`).join('');

  const tagRow = $('#infoTagRow');
  if (item.tag && TAG[item.tag]) {
    tagRow.innerHTML = `<span class="info-tag" style="--c:${tagColor(item.tag)}"><i></i>${esc(TAG[item.tag].label)}</span>`
      + '<span style="font-size:10.5px;color:var(--ink-4)">Global tag · visible from every folder</span>';
  } else {
    tagRow.innerHTML = '<span class="info-tag none">No tag yet</span>';
  }

  $('#infoDates').textContent = `Modified ${(item.date || 'today').replace('Today, ', 'today at ')}`;
  hydrateIcons(hero);

  const rename = $('#infoRename');
  const tagBtn = $('#infoTag');
  const shareBtn = $('#infoShare');
  if (extra && extra.readonly) {
    [rename, tagBtn, shareBtn].forEach(button => { if (button) button.hidden = true; });
  } else {
    [rename, tagBtn, shareBtn].forEach(button => { if (button) button.hidden = false; });
  }
}

function openInfo(key) {
  const found = findItem(key);
  if (!found) return;
  state.infoKey = key;
  const location = found.parent
    ? found.parent.name
    : found.device
      ? found.device.name
      : (state.route.type === 'folder' ? getView().title : 'Pictures');
  renderInfo(found.item, location);
  openModal('info');
}

function openLocationInfo() {
  const view = getView();
  state.infoKey = null;
  const pseudo = {
    name: view.title,
    kindLabel: view.route.type === 'folder' ? 'Folder' : 'Location',
    icon: view.icon,
    size: fmt(view.bytes),
    date: 'Today, 09:41',
    items: new Array(Math.max(view.count, 0)),
    tag: null
  };
  renderInfo(pseudo, 'Workspace', { readonly: true });
  openModal('info');
}

/* ---------- share · open with ---------- */

function shareAppsFor(item) {
  const kind = previewKind(item);
  const apps = [];
  if (kind === 'image') apps.push({ id: 'viewer', name: 'Smart Image Viewer', sub: 'Open in the viewer', icon: 'gallery', action: 'preview' });
  if (kind === 'audio') apps.push({ id: 'daw', name: 'Smart DAW', sub: 'Open the session', icon: 'music-notes', action: 'preview' });
  if (kind === 'doc') apps.push({ id: 'reader', name: 'Smart Reader', sub: 'Open the document', icon: 'document-text', action: 'preview' });
  if (kind === 'vector') apps.push({ id: 'vector', name: 'Smart Vector', sub: 'Edit the artwork', icon: 'stars-minimalistic', action: 'preview' });
  if (kind === 'archive') apps.push({ id: 'archive', name: 'Smart Archive', sub: 'Browse the contents', icon: 'archive', action: 'preview' });
  if (kind === 'app') apps.push({ id: 'launcher', name: 'Smart Launcher', sub: 'Run the app', icon: 'widget', action: 'preview' });
  apps.push({ id: 'suite', name: 'Smart Suite', sub: 'Workspace', icon: 'widget', action: 'receipt' });
  apps.push({ id: 'mail', name: 'Mail', sub: 'As an attachment', icon: 'arrow-right-up', action: 'receipt' });
  apps.push({ id: 'link', name: 'Copy link', sub: 'smart:// address', icon: 'copy', action: 'link' });
  return apps;
}

function openShare(key, mode = 'share') {
  const found = findItem(key);
  if (!found) return;
  state.shareKey = key;
  state.shareMode = mode;
  const item = found.item;

  $('#shareTitle').textContent = mode === 'openwith' ? 'Open with' : `Share “${item.name}”`;
  $('#shareSub').textContent = mode === 'openwith'
    ? 'Choose an app to open this file with'
    : 'Send this file to an app on this device';
  $('#shareNote').textContent = mode === 'openwith'
    ? 'The file stays in Smart Files until you choose an app.'
    : 'Nothing leaves this device until you send it.';

  const body = $('#shareBody');
  body.innerHTML = `<div class="share-grid">${shareAppsFor(item).map(app =>
    `<button class="share-dest" data-share-app="${app.id}">
      <span class="orb"><span class="ic" data-ic="${app.icon}"></span></span>
      <b>${esc(app.name)}</b><small>${esc(app.sub)}</small>
    </button>`).join('')}</div>`;
  hydrateIcons(body);
  openModal('share');
}

function shareReceipt(app, item) {
  const body = $('#shareBody');
  const title = app.id === 'link' ? 'Link copied' : `Shared with ${app.name}`;
  const copy = app.id === 'link'
    ? 'The smart:// address for this file is on your clipboard.'
    : `${item.name} is now available in ${app.name} on this device.`;
  body.innerHTML = `<div class="share-receipt">
    <span class="orb"><span class="ic" data-ic="check-circle"></span></span>
    <h4>${esc(title)}</h4>
    <p>${esc(copy)}</p>
    <div class="receipt-meta">
      <span class="cmd-chip">${esc(item.name)}</span>
      <span class="cmd-chip">${esc(app.name)}</span>
      <span class="cmd-chip">Just now</span>
    </div>
  </div>`;
  hydrateIcons(body);
  toast(title, null, null, 'check-circle');
}

function shareToApp(appId) {
  const found = state.shareKey ? findItem(state.shareKey) : null;
  if (!found) return;
  const item = found.item;
  const app = shareAppsFor(item).find(entry => entry.id === appId);
  if (!app) return;

  if (app.action === 'preview') {
    closeModals();
    openPreview(state.shareKey);
    toast(`Opened in ${app.name}`, null, null, app.icon);
    if (app.id === 'daw') setTimeout(() => toggleAudioPlayback(), 220);
    return;
  }

  if (app.action === 'link') {
    const link = `smart://files/${encodeURIComponent(item.name)}`;
    if (navigator.clipboard && navigator.clipboard.writeText) navigator.clipboard.writeText(link).catch(() => {});
    state.clipboard = state.shareKey;
    shareReceipt(app, item);
    return;
  }

  shareReceipt(app, item);
}

/* ---------- wallpaper ---------- */

function setWallpaper(id) {
  const wallpaper = WALLPAPERS.find(entry => entry.id === id) || WALLPAPERS[0];
  state.wallpaper = wallpaper.id;
  document.documentElement.style.setProperty('--wall-base', wallpaper.base);
  document.documentElement.style.setProperty('--wall-image', wallpaper.image);
  renderWallpaper();
  toast(`Wallpaper · ${wallpaper.name}`, null, null, 'gallery-wide');
}

function renderWallpaper() {
  const grid = $('#wallGrid');
  grid.innerHTML = WALLPAPERS.map(entry =>
    `<button class="wall-opt${state.wallpaper === entry.id ? ' on' : ''}" data-wall="${entry.id}">
      <span class="sw" style="background:${entry.image}"></span>
      <span class="cap">${esc(entry.name)}<small style="font-size:9.5px;color:var(--ink-4)">${esc(entry.sub)}</small></span>
      ${state.wallpaper === entry.id ? '<span class="check"><span class="ic" data-ic="check-circle"></span></span>' : ''}
    </button>`).join('');
  hydrateIcons(grid);
}

function openWallpaper() {
  renderWallpaper();
  openModal('wallpaper');
}

/* ---------- about · shortcuts ---------- */

function openAbout() {
  const links = [
    { id: 'tokens', icon: 'widget', title: 'Design tokens', note: 'smart.toml v0.1' },
    { id: 'shortcuts', icon: 'list-check', title: 'Keyboard shortcuts', note: `${SHORTCUTS.length} bindings` },
    { id: 'wallpaper', icon: 'gallery-wide', title: 'Desktop wallpaper', note: WALLPAPERS.find(entry => entry.id === state.wallpaper).name },
    { id: 'devices', icon: 'ssd-square', title: 'Devices & locations', note: `${state.devices.filter(d => d.mounted).length} mounted` },
    { id: 'phone', icon: 'users-group-rounded', title: 'Phone continuity', note: 'Naledzi’s Pixel' }
  ];
  const body = $('#aboutLinks');
  body.innerHTML = links.map(link =>
    `<button class="about-link" data-about="${link.id}">
      <span class="ic" data-ic="${link.icon}"></span>
      <b>${esc(link.title)}</b><small>${esc(link.note)}</small>
    </button>`).join('');
  hydrateIcons(body);

  const laws = $('#aboutLaws');
  if (laws) {
    laws.innerHTML = QUALITY_LAWS.map(law =>
      `<div class="about-law"><span class="ic" data-ic="check-circle"></span><span>${esc(law)}</span></div>`).join('');
    hydrateIcons(laws);
  }
  openModal('about');
}

function openShortcuts() {
  $('#scList').innerHTML = SHORTCUTS.map(([label, keys]) =>
    `<div class="sc-row"><span>${esc(label)}</span><kbd>${esc(keys)}</kbd></div>`).join('');
  openModal('shortcuts');
}

/* ---------- window chrome ---------- */

function showRestoreChip(label) {
  $('#restoreLabel').textContent = label;
  $('#restoreChip').classList.add('show');
}

function restoreWindow() {
  const win = $('#win');
  win.classList.remove('is-hidden');
  win.classList.add('is-restoring');
  setTimeout(() => win.classList.remove('is-restoring'), 540);
  $('#restoreChip').classList.remove('show');
  toast('Smart Files restored', null, null, 'refresh');
}

function newSmartWindow() {
  const opened = window.open(location.href.split('#')[0], '_blank', 'popup,width=1280,height=860');
  if (opened) toast('Opened a second Smart Files window', null, null, 'maximize');
  else toast('Allow pop-ups to open a second window', null, null, 'info-circle');
}

/* ---------- preview kinds · audio · document · app ---------- */

function previewKind(item) {
  if (!item) return null;
  if (item.thumb) return 'image';
  if (item.kind === 'Audio' || item.kind === 'Project') return 'audio';
  if (item.kind === 'Text' || item.kind === 'PDF') return 'doc';
  if (item.kind === 'Vector') return 'vector';
  if (item.kind === 'Archive') return 'archive';
  if (item.kind === 'App') return 'app';
  return null;
}

function archiveStageMarkup(item) {
  const entries = item.contents || [
    { name: 'brand-master.ai', size: '182 MB' },
    { name: 'wordmark.svg', size: '12 KB' },
    { name: 'palette.pdf', size: '1.4 MB' },
    { name: 'README.txt', size: '2 KB' }
  ];
  return `<div class="pv-stage archive">
    <div class="pv-paper">
      <h5>${esc(item.name)}</h5>
      <div class="pv-manifest">${entries.map(entry =>
        `<span class="pv-mrow"><span class="ic" data-ic="file"></span><span>${esc(entry.name)}</span><small>${esc(entry.size)}</small></span>`).join('')}</div>
      <p class="pv-mnote">${esc(item.size || '')} · packed locally, expands on demand</p>
    </div>
  </div>`;
}

function vectorStageMarkup(item) {
  const art = item.svg || '<svg viewBox="0 0 220 64" xmlns="http://www.w3.org/2000/svg"><rect width="220" height="64" rx="8" fill="#eef2f8"/><text x="110" y="40" text-anchor="middle" font-family="sans-serif" font-size="26" font-weight="600" fill="#141d30">NALEDZI</text></svg>';
  return `<div class="pv-stage vector">
    <div class="pv-vector-card">
      <div class="pv-vector-art">${art}</div>
      <div class="pv-vector-meta">
        <b>${esc(item.name)}</b>
        <small>${esc(item.kindLabel || 'SVG vector')} · ${esc(item.size || '')}</small>
        <small>${esc(item.dims || 'scalable')}</small>
      </div>
    </div>
  </div>`;
}

function audioDuration(item) {
  const bytes = item && item.bytes ? item.bytes : 0;
  return Math.max(4, Math.round(bytes / 176400));
}

function audioBpm(item) {
  const match = /(\d{2,3})\s*BPM/i.exec(item.name || '');
  return match ? `${match[1]} BPM` : '126 BPM';
}

function waveBars(key) {
  let seed = 7;
  for (let i = 0; i < key.length; i += 1) seed = (seed * 31 + key.charCodeAt(i)) % 9973;
  const bars = [];
  for (let i = 0; i < 52; i += 1) {
    const a = Math.sin((i + seed % 11) * 0.52) * 0.5 + 0.5;
    const b = Math.sin((i * 1.73 + seed % 17) * 0.41) * 0.5 + 0.5;
    const height = 9 + Math.round((a * 0.62 + b * 0.38) * 40);
    const y = Math.round(32 - height / 2);
    bars.push(`<rect x="${i * 4}" y="${y}" width="2" height="${height}" rx="1"/>`);
  }
  return bars.join('');
}

function audioStageMarkup(item) {
  const duration = audioDuration(item);
  return `<div class="pv-audio">
    <div class="pv-wave" id="pvWave">
      <svg class="base" viewBox="0 0 208 64" preserveAspectRatio="none" aria-hidden="true"><g fill="currentColor">${waveBars(item.key)}</g></svg>
      <svg class="live" viewBox="0 0 208 64" preserveAspectRatio="none" aria-hidden="true"><g fill="currentColor">${waveBars(item.key)}</g></svg>
    </div>
    <div class="pv-transport">
      <button class="pv-play" id="pvPlay" aria-label="Play"><i class="tri"></i></button>
      <span class="pv-time" id="pvTime">0:00</span>
      <div class="pv-track"><i id="pvFill"></i></div>
      <span class="pv-dur" id="pvDur">${fmtTime(duration)}</span>
      <span class="pv-bpm">${esc(audioBpm(item))}</span>
    </div>
  </div>`;
}

function docStageMarkup(item) {
  const content = item.content || `${item.name}\n\nThis file carries no inline preview yet.`;
  return `<div class="pv-paper"><h5>${esc(item.name)}</h5><pre>${esc(content)}</pre></div>`;
}

function appStageMarkup(item) {
  return `<div class="pv-app">
    <span class="pv-app-orb"><span class="ic" data-ic="${item.icon || 'widget'}"></span></span>
    <div class="pv-app-name">${esc(item.name)}</div>
    <div class="pv-app-meta">${esc(item.kindLabel || 'App')} · ${esc(item.size || '')} · ${esc(item.date || '')}</div>
    <div class="pv-app-progress"><i id="pvAppFill"></i></div>
    <div class="pv-app-status" id="pvAppStatus">Ready to launch</div>
  </div>`;
}

function stopAudioPreview() {
  if (audioTimer) clearInterval(audioTimer);
  audioTimer = null;
  state.audioElapsed = 0;
  const button = $('#pvPlay');
  if (button) {
    button.classList.remove('playing');
    button.setAttribute('aria-label', 'Play');
  }
}

function updateAudioUI(elapsed, duration) {
  const percent = Math.min(100, (elapsed / duration) * 100);
  const wave = $('#pvWave');
  if (wave) {
    const live = wave.querySelector('.live');
    if (live) live.style.clipPath = `inset(0 ${100 - percent}% 0 0)`;
  }
  const fill = $('#pvFill');
  if (fill) fill.style.width = `${percent}%`;
  const time = $('#pvTime');
  if (time) time.textContent = fmtTime(elapsed);
}

function toggleAudioPlayback() {
  const button = $('#pvPlay');
  if (!button) return;
  if (audioTimer) {
    stopAudioPreview();
    return;
  }
  const found = findItem(state.previewKey);
  if (!found) return;
  const duration = audioDuration(found.item);
  button.classList.add('playing');
  button.setAttribute('aria-label', 'Pause');
  const startedAt = Date.now() - (state.audioElapsed || 0) * 1000;
  audioTimer = setInterval(() => {
    let elapsed = (Date.now() - startedAt) / 1000;
    if (elapsed >= duration) elapsed = 0;
    state.audioElapsed = elapsed;
    updateAudioUI(elapsed, duration);
  }, 100);
}

function seekAudio(event) {
  const wave = event.target.closest('#pvWave');
  if (!wave) return;
  const found = findItem(state.previewKey);
  if (!found) return;
  const rect = wave.getBoundingClientRect();
  const ratio = Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width));
  const duration = audioDuration(found.item);
  state.audioElapsed = ratio * duration;
  updateAudioUI(state.audioElapsed, duration);
  if (!audioTimer) toggleAudioPlayback();
}

function stopAppPreview() {
  if (appTimer) clearInterval(appTimer);
  appTimer = null;
}

function launchApp(item) {
  const fill = $('#pvAppFill');
  const status = $('#pvAppStatus');
  if (!fill || !status) return;
  if (appTimer) clearInterval(appTimer);
  let progress = 0;
  status.textContent = 'Launching…';
  appTimer = setInterval(() => {
    progress = Math.min(100, progress + 14);
    fill.style.width = `${progress}%`;
    if (progress >= 100) {
      clearInterval(appTimer);
      appTimer = null;
      status.textContent = 'Ready — use Reveal to open the SMART-DAW Beta folder';
      toast('smart-daw is ready to open', null, null, 'widget');
    }
  }, 110);
}
/* ============================================================
   devices · mounting · transfer · continuity
   ============================================================ */

const TRANSFER_PLACES = [
  { id: 'pictures', name: 'Pictures', icon: 'gallery', kind: 'local' },
  { id: 'documents', name: 'Documents', icon: 'document-text', kind: 'local' },
  { id: 'downloads', name: 'Downloads', icon: 'download-minimalistic', kind: 'local' },
  { id: 'drive', name: 'Smart Drive', icon: 'ssd-square', kind: 'device', device: 'drive' },
  { id: 'sanDisk', name: 'SanDisk Ultra', icon: 'ssd-square', kind: 'device', device: 'sanDisk' },
  { id: 'nas', name: 'Studio NAS', icon: 'global', kind: 'device', device: 'nas' },
  { id: 'phone', name: 'Naledzi’s Pixel', icon: 'users-group-rounded', kind: 'phone' }
];

const QUALITY_LAWS = [
  'Glass never carries text on its own — tonal surfaces do.',
  'Five elevation levels, tonal steps instead of decorative strokes.',
  'Solar icons only: outline by default, duotone on hover, filled when active.',
  'Every text pair passes WCAG AA; focus is always visible.',
  'Touch targets reach 44 px; reduced motion is honoured.',
  'One smart.toml — spec-perfect parity, never pixel-perfect.'
];

function deviceById(id) {
  return state.devices.find(device => device.id === id);
}

function deviceFiles(id) {
  const device = deviceById(id);
  if (!device || !device.mounted) return [];
  return device.files || [];
}

function devicePlace(deviceId) {
  return TRANSFER_PLACES.find(place => place.device === deviceId);
}

/* ---------- sidebar ---------- */

function renderSidebarDevices() {
  const mount = $('#sbDevices');
  if (!mount) return;
  const html = state.devices.map(device => {
    const icon = device.icon || 'ssd-square';
    const tail = device.mounted
      ? (device.kind === 'Continuity' ? 'Paired' : 'Mounted')
      : 'Ejected';
    const active = state.route.type === 'device' && state.route.id === device.id;
    return `<button class="row dev-row-sb${device.mounted ? '' : ' ejected'}${active ? ' active' : ''}" data-device="${device.id}">
      <span class="ic" data-ic="${icon}"></span><span class="devname">${esc(device.name)}</span><span class="devdot"></span><span class="tail">${tail}</span>
    </button>`;
  }).join('');
  if (mount.dataset.sig === html) return;
  mount.dataset.sig = html;
  mount.innerHTML = html;
  hydrateIcons(mount);
}

/* ---------- devices ---------- */

function renderDevices() {
  const list = $('#devList');
  if (!list) return;
  list.innerHTML = state.devices.map(device => {
    const tone = device.tone === 'amber' ? 'var(--warn)' : device.tone === 'green' ? 'var(--good)' : 'var(--accent)';
    const meter = device.mounted ? `<div class="dev-meter"><i style="width:${device.percent}%"></i></div>` : '';
    const stateChip = device.mounted
      ? '<span class="dev-state on"><i></i>Mounted</span>'
      : '<span class="dev-state"><i></i>Ejected</span>';
    const action = device.mounted
      ? `<button class="dev-act" data-dev-act="eject" data-device="${device.id}">Eject</button>
         <button class="dev-act primary" data-dev-act="open" data-device="${device.id}">Open</button>`
      : `<button class="dev-act primary" data-dev-act="mount" data-device="${device.id}">Mount</button>`;
    return `<div class="dev-row${device.mounted ? '' : ' ejected'}" data-device-row="${device.id}">
      <span class="dev-orb" style="--tone:${tone}"><span class="ic" data-ic="${device.icon || 'ssd-square'}"></span></span>
      <span class="dev-meta">
        <b>${esc(device.name)}</b>
        <small>${esc(device.kind)} · ${esc(device.capacity)}</small>
        ${meter}
      </span>
      ${stateChip}
      ${action}
    </div>`;
  }).join('');
  hydrateIcons(list);
}

function openDevices() {
  renderDevices();
  openModal('devices');
}

function mountDevice(id) {
  const device = deviceById(id);
  if (!device || device.mounted) return;
  const row = $(`[data-device-row="${id}"]`);
  const button = row ? row.querySelector('[data-dev-act="mount"]') : null;
  if (button) {
    button.className = 'dev-mount';
    button.innerHTML = '<svg class="throbber" viewBox="0 0 24 24" aria-hidden="true"><circle class="trk" cx="12" cy="12" r="9"></circle><circle class="bar" cx="12" cy="12" r="9"></circle></svg><span>Mounting</span>';
  }
  setTimeout(() => {
    device.mounted = true;
    device.percent = device.percent || 12;
    renderDevices();
    renderSidebarDevices();
    render();
    toast(`${device.name} mounted`, 'Open', () => navigate({ type: 'device', id: device.id }), 'ssd-square');
  }, 950);
}

function ejectDevice(id) {
  const device = deviceById(id);
  if (!device || !device.mounted) return;
  confirmAction({
    title: `Eject ${device.name}?`,
    body: `${device.capacity} will disappear from the sidebar until you mount it again.`,
    ok: 'Eject'
  }, () => {
    device.mounted = false;
    renderDevices();
    renderSidebarDevices();
    if (state.route.type === 'device' && state.route.id === id) navigate({ type: 'nav', id: 'pictures' });
    else render();
    toast(`${device.name} ejected`, 'Remount', () => mountDevice(id), 'ssd-square');
  });
}

function openDevice(id) {
  const device = deviceById(id);
  if (!device) return;
  if (!device.mounted) {
    mountDevice(id);
    return;
  }
  if (device.kind === 'Continuity') {
    openPhone();
    return;
  }
  navigate({ type: 'device', id });
}

/* ---------- confirm ---------- */

function confirmAction({ title, body, ok = 'Confirm' }, onConfirm) {
  state.confirmAction = onConfirm;
  $('#cfTitle').textContent = title;
  $('#cfBody').textContent = body;
  $('#cfOk').textContent = ok;
  openModal('confirm');
}

/* ---------- transfer · copy & move ---------- */

function transferDestinations(placeId) {
  const place = TRANSFER_PLACES.find(entry => entry.id === placeId);
  if (!place) return [];
  if (place.kind === 'local') {
    if (place.id === 'pictures') {
      return [
        { id: 'pictures-root', name: 'Pictures', icon: 'gallery', sub: 'Top level', count: `${FILES.filter(file => !state.trash.has(file.key)).length + FOLDERS.filter(folder => !state.trash.has(folder.key)).length} items` },
        { id: 'pictures-amapiano', name: 'Amapiano Pack Vol. 1', icon: 'folder', sub: 'Audio + artwork', count: '4 items', folder: 'amapiano' },
        { id: 'pictures-brand', name: 'Brand — Naledzi', icon: 'folder', sub: 'Identity files', count: '3 items', folder: 'brand' },
        { id: 'pictures-daw', name: 'SMART-DAW Beta', icon: 'folder', sub: 'Build + layout', count: '3 items', folder: 'daw' }
      ];
    }
    if (place.id === 'documents') {
      return [{ id: 'documents-root', name: 'Documents', icon: 'document-text', sub: 'Project files', count: '4 items' }];
    }
    return [{ id: 'downloads-root', name: 'Downloads', icon: 'download-minimalistic', sub: 'Incoming files', count: '0 items' }];
  }
  if (place.kind === 'device') {
    const device = deviceById(place.device);
    const files = deviceFiles(place.device);
    const destinations = [{ id: `${place.device}-root`, name: device.name, icon: device.icon, sub: device.kind, count: `${files.length} items`, device: place.device }];
    if (place.device === 'nas') {
      destinations.push({ id: 'nas-backups', name: 'Session Backups', icon: 'folder', sub: 'Auto-synced', count: '2 items', device: 'nas', subFolder: 'backups' });
    }
    if (place.device === 'sanDisk') {
      destinations.push({ id: 'sanDisk-pack', name: 'Amapiano Pack', icon: 'folder', sub: 'Release staging', count: '1 item', device: 'sanDisk', subFolder: 'pack' });
    }
    return destinations;
  }
  return [
    { id: 'phone-recents', name: 'Recents', icon: 'clock-circle', sub: 'On Naledzi’s Pixel', count: `${state.phoneFiles.length} items` },
    { id: 'phone-downloads', name: 'Downloads', icon: 'download-minimalistic', sub: 'Saved on the phone', count: '0 items' }
  ];
}

function transferSourceContainer(item) {
  const index = FILES.findIndex(file => file.key === item.key);
  if (index >= 0) return { list: FILES, index, kind: 'files' };
  const folderIndex = FOLDERS.findIndex(folder => folder.key === item.key);
  if (folderIndex >= 0) return { list: FOLDERS, index: folderIndex, kind: 'folders' };
  for (const folder of FOLDERS) {
    const inner = folder.items.findIndex(entry => entry.key === item.key);
    if (inner >= 0) return { list: folder.items, index: inner, kind: 'inner', parent: folder };
  }
  return null;
}

function transferDestinationContainer(destination) {
  if (destination.device) {
    const device = deviceById(destination.device);
    if (!device) return null;
    if (!device.files) device.files = [];
    if (destination.subFolder === 'backups') {
      if (!device.backups) device.backups = [];
      return { list: device.backups, kind: 'device', label: 'Session Backups' };
    }
    if (destination.subFolder === 'pack') {
      if (!device.pack) device.pack = [];
      return { list: device.pack, kind: 'device', label: 'Amapiano Pack' };
    }
    return { list: device.files, kind: 'device', label: device.name };
  }
  if (destination.folder) {
    const folder = FOLDERS.find(entry => entry.id === destination.folder);
    if (folder) return { list: folder.items, kind: 'folder', label: folder.name };
  }
  if (destination.id.startsWith('phone')) {
    return { list: state.phoneFiles, kind: 'phone', label: 'Naledzi’s Pixel' };
  }
  if (destination.id === 'documents-root') return { list: state.documentsFiles, kind: 'documents', label: 'Documents' };
  if (destination.id === 'downloads-root') return { list: state.downloadsFiles, kind: 'downloads', label: 'Downloads' };
  return { list: FILES, kind: 'files', label: 'Pictures' };
}

function beginTransfer(mode, key) {
  const found = findItem(key);
  if (!found) return;
  const place = TRANSFER_PLACES[0];
  const first = transferDestinations(place.id)[0];
  state.transfer = { mode, key, place: place.id, dest: first ? first.id : null, phase: 'idle' };
  openModal('transfer');
  renderTransfer();
}

function renderTransfer() {
  const transfer = state.transfer;
  if (!transfer) return;
  const found = findItem(transfer.key);
  if (!found) return;
  const item = found.item;
  const place = TRANSFER_PLACES.find(entry => entry.id === transfer.place) || TRANSFER_PLACES[0];
  const device = place.device ? deviceById(place.device) : null;

  $('#trTitle').textContent = transfer.mode === 'move' ? 'Move to…' : 'Copy to…';
  $('#trSub').textContent = transfer.mode === 'move'
    ? `Move “${item.name}” somewhere else`
    : `Duplicate “${item.name}” somewhere else`;

  const rail = $('#trRail');
  rail.innerHTML = TRANSFER_PLACES.map(entry => {
    const entryDevice = entry.device ? deviceById(entry.device) : null;
    const ejected = entryDevice && !entryDevice.mounted;
    const label = entryDevice ? entryDevice.name : entry.name;
    return `<button class="tr-place${entry.id === transfer.place ? ' active' : ''}${ejected ? ' ejected' : ''}" data-tr-place="${entry.id}">
      <span class="ic" data-ic="${entryDevice ? entryDevice.icon : entry.icon}"></span>
      <span>${esc(label)}</span>
      ${ejected ? '<small>Mount</small>' : ''}
    </button>`;
  }).join('');
  hydrateIcons(rail);

  $('#trPath').innerHTML = `<span class="ic" data-ic="${place.icon}" style="--ic:13px"></span><b>${esc(device ? device.name : place.name)}</b><span>›</span><span>${esc(place.kind === 'local' ? 'This device' : place.kind === 'phone' ? 'Continuity' : 'External')}</span>`;
  hydrateIcons($('#trPath'));

  const list = $('#trList');
  list.innerHTML = transferDestinations(transfer.place).map(dest => `
    <button class="tr-row${dest.id === transfer.dest ? ' on' : ''}" data-tr-dest="${dest.id}">
      <span class="ic" data-ic="${dest.icon}"></span>
      <span><b>${esc(dest.name)}</b><small>${esc(dest.sub)}</small></span>
      <span class="tr-count">${esc(dest.count || '')}</span>
    </button>`).join('');
  hydrateIcons(list);

  const thumb = item.thumb
    ? `<span class="tr-thumb"><img src="assets/thumbs/${item.thumb}.jpg" alt=""></span>`
    : `<span class="tr-thumb"><span class="ic" data-ic="${item.icon || 'file'}"></span></span>`;
  $('#trSide').innerHTML = `
    <div class="tr-chip" id="trChip">${thumb}<span style="min-width:0"><b>${esc(item.name)}</b><small>${esc(item.kindLabel || 'File')} · ${esc(item.size || '')}</small></span></div>
    <div class="tr-mode">
      <button class="${transfer.mode === 'copy' ? 'on' : ''}" data-tr-mode="copy">Copy</button>
      <button class="${transfer.mode === 'move' ? 'on' : ''}" data-tr-mode="move">Move</button>
    </div>
    <p class="tr-note">${transfer.mode === 'move'
      ? 'The file leaves its current folder once the hand-off lands.'
      : 'A duplicate is created — the original stays where it is.'}</p>
    <button class="tr-go" id="trGo"${transfer.phase === 'done' ? ' disabled' : ''}>${transfer.phase === 'done' ? 'Transferred' : transfer.mode === 'move' ? 'Move here' : 'Copy here'}</button>`;
  hydrateIcons($('#trSide'));
}

function selectTransferPlace(placeId) {
  if (!state.transfer) return;
  const place = TRANSFER_PLACES.find(entry => entry.id === placeId);
  if (place && place.device) {
    const device = deviceById(place.device);
    if (device && !device.mounted) {
      mountDevice(place.device);
      state.transfer.place = placeId;
      state.transfer.dest = null;
      renderTransfer();
      return;
    }
  }
  state.transfer.place = placeId;
  const first = transferDestinations(placeId)[0];
  state.transfer.dest = first ? first.id : null;
  state.transfer.phase = 'idle';
  renderTransfer();
}

function flyChip(fromEl, toEl, onDone) {
  if (!fromEl || !toEl || window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    if (onDone) onDone();
    return;
  }
  const from = fromEl.getBoundingClientRect();
  const to = toEl.getBoundingClientRect();
  const chip = fromEl.cloneNode(true);
  chip.classList.add('flying-chip');
  document.body.appendChild(chip);
  chip.style.left = `${from.left}px`;
  chip.style.top = `${from.top}px`;
  chip.style.width = `${from.width}px`;
  chip.style.height = `${from.height}px`;
  const dx = to.left + to.width / 2 - (from.left + from.width / 2);
  const dy = to.top + to.height / 2 - (from.top + from.height / 2);
  const animation = chip.animate([
    { transform: 'translate(0,0) scale(1)', opacity: 1 },
    { transform: `translate(${dx * 0.42}px, ${dy * 0.42 - 54}px) scale(1.08) rotate(-2deg)`, opacity: 1, offset: 0.55 },
    { transform: `translate(${dx}px, ${dy}px) scale(.34)`, opacity: 0.1 }
  ], { duration: 700, easing: 'cubic-bezier(.34,1.42,.44,1)', fill: 'forwards' });
  animation.onfinish = () => {
    chip.remove();
    if (onDone) onDone();
  };
}

function runTransfer(destinationId) {
  const transfer = state.transfer;
  if (!transfer || transfer.phase !== 'idle') return;
  const targetId = destinationId || transfer.dest;
  if (!targetId) {
    toast('Choose where the file should land', null, null, 'info-circle');
    return;
  }
  const found = findItem(transfer.key);
  if (!found) return;
  const item = found.item;
  const destination = transferDestinations(transfer.place).find(entry => entry.id === targetId);
  if (!destination) return;

  const target = transferDestinationContainer(destination);
  if (!target) return;

  const source = transferSourceContainer(item);
  const sameContainer = source && source.list === target.list;
  if (sameContainer) {
    toast(`“${item.name}” is already in ${target.label}`, null, null, 'info-circle');
    return;
  }

  const row = $(`[data-tr-dest="${targetId}"]`);
  const chip = $('#trChip');
  transfer.phase = 'flying';
  if (row) {
    const ring = document.createElement('svg');
    ring.setAttribute('class', 'throbber');
    ring.setAttribute('viewBox', '0 0 24 24');
    ring.innerHTML = '<circle class="trk" cx="12" cy="12" r="9"></circle><circle class="bar" cx="12" cy="12" r="9"></circle>';
    row.appendChild(ring);
  }

  flyChip(chip, row, () => {
    if (row) {
      row.classList.add('receiving');
      const ring = row.querySelector('.throbber');
      if (ring) setTimeout(() => ring.remove(), 420);
    }

    setTimeout(() => {
      const stamp = Date.now();
      const copy = {
        ...item,
        key: `${item.key}-${transfer.mode}-${stamp}`,
        name: transfer.mode === 'copy' ? copyName(item.name) : item.name,
        date: 'Just now'
      };
      if (item.items) copy.items = item.items.map(entry => ({ ...entry, key: `${entry.key}-${transfer.mode}-${stamp}` }));
      target.list.unshift(copy);

      if (transfer.mode === 'move' && source) {
        source.list.splice(source.index, 1);
      }

      transfer.phase = 'done';
      render();
      renderSidebarDevices();
      const verb = transfer.mode === 'move' ? 'Moved' : 'Copied';
      $('#trGo').textContent = 'Transferred';
      $('#trGo').disabled = true;
      $('#trNote').textContent = `${verb} to ${target.label}.`;
      toast(`${verb} “${item.name}” to ${target.label}`, 'Undo', () => {
        const undoIndex = target.list.findIndex(entry => entry.key === copy.key);
        if (undoIndex >= 0) target.list.splice(undoIndex, 1);
        if (transfer.mode === 'move' && source) source.list.splice(source.index, 0, item);
        render();
      }, transfer.mode === 'move' ? 'folder' : 'copy');
    }, 430);
  });
}

/* ---------- phone continuity ---------- */

function renderPhone() {
  const device = state.devices.find(entry => entry.kind === 'Continuity');
  const connected = device && device.mounted;
  const status = $('#phStatus');
  status.innerHTML = `<span class="ph-orb"><span class="ic" data-ic="users-group-rounded"></span></span>
    <span><b>${esc(device ? device.name : 'Naledzi’s Pixel')}</b><small>${connected ? 'Connected over Wi-Fi · 42% battery' : 'Not connected'}</small></span>`;

  const clipboard = state.clipboard ? findItem(state.clipboard) : null;
  const list = $('#phList');
  list.innerHTML = `
    <button class="ph-row" data-ph-act="clipboard">
      <span class="ic" data-ic="copy"></span>
      <span><b>Universal clipboard</b><small>Copy on desktop, paste on the phone</small></span>
      <span class="ph-chip">${clipboard ? esc(clipboard.item.name) : 'Empty'}</span>
    </button>
    <button class="ph-row" data-ph-act="handover">
      <span class="ic" data-ic="share"></span>
      <span><b>Handover</b><small>Continue the selection on the phone</small></span>
      <span class="ph-chip">${state.selection.size ? `${state.selection.size} selected` : 'Nothing'}</span>
    </button>
    <button class="ph-row" data-ph-act="relay">
      <span class="ic" data-ic="text"></span>
      <span><b>Text relay</b><small>Type on the phone, land in this folder</small></span>
      <span class="ph-chip">Ready</span>
    </button>`;
  hydrateIcons(list);

  const received = $('#phReceived');
  if (state.phoneFiles.length) {
    received.innerHTML = state.phoneFiles.map(file => `
      <div class="ph-row"><span class="ic" data-ic="${file.thumb ? 'gallery' : 'file'}"></span>
      <span><b>${esc(file.name)}</b><small>Received ${esc(file.date || 'just now')}</small></span></div>`).join('');
    hydrateIcons(received);
  } else {
    received.innerHTML = '';
  }
}

function openPhone() {
  renderPhone();
  openModal('phone');
}

function sendToPhone(item) {
  if (!item) return;
  const found = findItem(item.key);
  if (!found) return;
  const copy = { ...found.item, key: `${found.item.key}-phone-${Date.now()}`, date: 'Just now' };
  state.phoneFiles.unshift(copy);
  renderPhone();
  toast(`Sent “${copy.name}” to Naledzi’s Pixel`, 'Open', openPhone, 'users-group-rounded');
}

function phoneAction(action) {
  if (action === 'clipboard') {
    if (!state.clipboard) {
      toast('Copy something first, then paste it on the phone', null, null, 'copy');
      return;
    }
    const found = findItem(state.clipboard);
    sendToPhone(found.item);
    return;
  }
  if (action === 'handover') {
    const selected = [...state.selection][0];
    if (!selected) {
      toast('Select a file first, then hand it over', null, null, 'share');
      return;
    }
    const found = findItem(selected);
    sendToPhone(found.item);
    return;
  }
  if (action === 'relay') {
    const note = {
      key: `phone-note-${Date.now()}`,
      name: `Relay note ${state.phoneFiles.length + 1}.txt`,
      kind: 'Text', kindLabel: 'Plain text', ext: 'TXT', icon: 'text',
      size: '2 KB', bytes: 2000, date: 'Just now', ts: 101, tag: 'archive',
      content: 'Typed on Naledzi’s Pixel and relayed to this folder.'
    };
    state.phoneFiles.unshift(note);
    renderPhone();
    toast('Relayed a note from the phone', 'Open', openPhone, 'text');
  }
}

/* ============================================================
   actions
   ============================================================ */

function openKeyMenu(key, x, y) {
  const found = findItem(key);
  if (!found) return;
  showMenu(fileMenu(found.item), x, y, key);
}

function openEmptyMenu(x, y) {
  showMenu(emptyMenuItems(), x, y);
}

function newFolder() {
  const count = FOLDERS.filter(f => f.name.startsWith('Untitled Folder')).length + 1;
  const id = `untitled-${Date.now()}`;
  const folder = {
    key: `fold-${id}`,
    id,
    type: 'folder',
    name: count === 1 ? 'Untitled Folder' : `Untitled Folder ${count}`,
    tag: 'design',
    ts: 110,
    kindLabel: 'Folder',
    date: 'Today, 09:41',
    items: []
  };
  FOLDERS.unshift(folder);

  if (!(state.route.type === 'nav' && state.route.id === 'pictures')) {
    navigate({ type: 'nav', id: 'pictures' });
  } else {
    render();
  }
  toast('New folder created', 'Open', () => navigate({ type: 'folder', id }));
}

function trashKey(key) {
  const found = findItem(key);
  if (!found) return;
  state.trash.add(key);
  state.selection.delete(key);
  render();
  toast(`Moved “${found.item.name}” to Trash`, 'Undo', () => {
    state.trash.delete(key);
    render();
  }, 'trash-bin-minimalistic');
}

function trashSelection() {
  const keys = [...state.selection];
  if (!keys.length) return;
  if (keys.length === 1) {
    trashKey(keys[0]);
    return;
  }
  keys.forEach(key => state.trash.add(key));
  state.selection.clear();
  render();
  toast(`${keys.length} items moved to Trash`, 'Undo', () => {
    keys.forEach(key => state.trash.delete(key));
    render();
  }, 'trash-bin-minimalistic');
}

function executeMenu(item, key) {
  const pos = state.lastMenuPos || { x: 40, y: 40 };
  switch (item.action) {
    case 'open':
      if (key) openKey(key);
      break;
    case 'preview':
      if (key) openPreview(key);
      break;
    case 'openwith':
      if (key) openShare(key, 'openwith');
      else toast(`Opening with ${item.app}…`, null, null, 'arrow-right-up');
      break;
    case 'share':
      if (key) openShare(key, 'share');
      break;
    case 'copy':
      if (key) copyItem(key);
      else copySelection();
      break;
    case 'copyto':
      if (key) beginTransfer('copy', key);
      else toast('Select something to copy', null, null, 'copy');
      break;
    case 'move':
      if (key) beginTransfer('move', key);
      else toast('Select something to move', null, null, 'folder');
      break;
    case 'paste':
      pasteItem();
      break;
    case 'rename':
      if (key) startRename(key);
      break;
    case 'tag':
      if (key) openTagPicker(key, pos.x, pos.y);
      break;
    case 'settag':
      if (key) setItemTag(key, item.value);
      break;
    case 'info':
      if (key) openInfo(key);
      else openLocationInfo();
      break;
    case 'locationinfo':
      openLocationInfo();
      break;
    case 'sendphone':
      if (key) sendToPhone(findItem(key) ? findItem(key).item : null);
      break;
    case 'trash':
      if (key) trashKey(key);
      break;
    case 'newfolder':
      newFolder();
      break;
    case 'sort':
      state.sort = item.value;
      render();
      break;
    case 'viewopts':
      openViewOptions(pos.x, pos.y);
      break;
    case 'opt-ext':
      toggleOption('ext');
      break;
    case 'opt-dates':
      toggleOption('dates');
      break;
    case 'opt-density':
      toggleOption('density');
      break;
    case 'gridoverlay':
      toggleGrid();
      break;
    case 'wallpaper':
      openWallpaper();
      break;
    case 'tokens':
      openModal('tokens');
      break;
    case 'theme':
      toggleTheme();
      break;
    case 'sidebar':
      toggleSidebar();
      break;
    case 'palette':
      openPalette();
      break;
    case 'newwindow':
      newSmartWindow();
      break;
    case 'about':
      openAbout();
      break;
    case 'shortcuts':
      openShortcuts();
      break;
    case 'devices':
      openDevices();
      break;
    case 'phone':
      openPhone();
      break;
    case 'mount-device':
      mountDevice(state.route.id);
      break;
    case 'workspace': {
      $('.ws-text b').textContent = item.value;
      $('.ws-text small').textContent = item.subvalue || '';
      toast(`Switched to ${item.value}`, null, null, 'users-group-rounded');
      break;
    }
    default:
      break;
  }
}

function handleTraffic(action) {
  const win = $('#win');
  if (win.dataset.busy) return;
  closeMenus();

  if (action === 'zoom') {
    win.classList.toggle('is-zoomed');
    return;
  }

  win.dataset.busy = '1';
  win.classList.add(action === 'close' ? 'is-closing' : 'is-minimizing');
  setTimeout(() => {
    win.classList.remove('is-closing', 'is-minimizing');
    delete win.dataset.busy;
    win.classList.add('is-hidden');
    showRestoreChip(action === 'close' ? 'Smart Files closed' : 'Smart Files minimised');
    if (action === 'close') {
      state.openModal = null;
      state.previewKey = null;
      stopAudioPreview();
      stopAppPreview();
      $('#scrim').classList.remove('show');
      $$('.modal').forEach(modal => modal.classList.remove('show'));
    }
  }, 720);
}

/* ============================================================
   search · view · hash state
   ============================================================ */

function setQuery(value) {
  state.query = value;
  const input = $('#searchInput');
  input.value = value;
  $('#searchBox').classList.toggle('has-val', !!value);
  state.selection.clear();
  render();
}

function clearSearch(focus = true) {
  setQuery('');
  if (focus) {
    const input = $('#searchInput');
    input.focus();
    input.select();
  }
}

function openSearch() {
  const input = $('#searchInput');
  input.focus();
  input.select();
}

function setView(view) {
  state.view = view;
  render();
}

function applyHash() {
  const raw = location.hash.replace(/^#/, '');
  if (raw) {
    const parts = raw.split('&').map(part => decodeURIComponent(part));
    if (parts.includes('still')) document.body.classList.add('still');
    if (parts.includes('dark')) setTheme('dark');
    if (parts.includes('collapsed')) setCollapsed(true);
    if (parts.includes('list')) state.view = 'list';

    const folderPart = parts.find(part => part.startsWith('folder='));
    if (folderPart) {
      const id = folderPart.split('=')[1];
      if (FOLDERS.some(folder => folder.id === id)) state.route = { type: 'folder', id };
    }

    const tagPart = parts.find(part => part.startsWith('tag='));
    if (tagPart) {
      const id = tagPart.split('=')[1];
      if (TAG[id]) state.route = { type: 'tag', id };
    }
  }

  render();

  if (!raw) return;
  const parts = raw.split('&').map(part => decodeURIComponent(part));

  if (parts.includes('menu')) {
    requestAnimationFrame(() => {
      const el = $('.card[data-key]') || $('.folder[data-key]');
      if (!el) return;
      const rect = el.getBoundingClientRect();
      openKeyMenu(el.dataset.key, rect.left + rect.width * 0.55, rect.top + 46);
    });
  }
  if (parts.includes('preview')) openPreview('img-frosted');
  if (parts.includes('tokens')) openModal('tokens');
  if (parts.includes('ask')) openAI();
  if (parts.includes('devices')) openDevices();
  if (parts.includes('phone')) openPhone();
  if (parts.includes('wallpaper')) openWallpaper();
  if (parts.includes('about')) openAbout();
  if (parts.includes('shortcuts')) openShortcuts();
  if (parts.includes('info')) openInfo('img-frosted');
  if (parts.includes('share')) openShare('img-frosted');
  if (parts.includes('transfer')) beginTransfer('copy', 'img-frosted');
  if (parts.includes('audio')) {
    navigate({ type: 'folder', id: 'amapiano' }, false);
    openPreview('file-logdrum-126');
  }
  if (parts.includes('vector')) {
    navigate({ type: 'folder', id: 'brand' }, false);
    openPreview('file-brand-svg');
  }
  if (parts.includes('archive')) {
    navigate({ type: 'device', id: 'nas' }, false);
    openPreview('nas-brand');
  }
  if (parts.includes('palette')) {
    const railPart = parts.find(part => part.startsWith('rail='));
    const rail = railPart ? railPart.split('=')[1] : 'all';
    requestAnimationFrame(() => openPalette(rail));
  }
  if (parts.includes('grid')) $('#gridOverlay').classList.add('on');
}

/* ============================================================
   bindings
   ============================================================ */

function bindEvents() {
  $('#win').addEventListener('click', event => {
    const win = event.currentTarget;
    const control = event.target.closest('[data-action]');
    if (control) {
      handleTraffic(control.dataset.action);
      return;
    }
    if (event.target === win
      && window.matchMedia('(max-width: 720px)').matches
      && !win.classList.contains('is-collapsed')) {
      setCollapsed(true);
    }
  });

  document.addEventListener('click', event => {
    const close = event.target.closest('[data-close]');
    if (close) {
      if (close.dataset.close === 'ai') closeAI();
      else closeModals();
      return;
    }

    if (event.target.id === 'scrim') {
      closeModals();
      return;
    }

    if (!event.target.closest('.menu')
      && !event.target.closest('#sortBtn')
      && !event.target.closest('#moreBtn')
      && !event.target.closest('#workspace')
      && !event.target.closest('[data-act="menu"]')) {
      closeMenus();
    }
  });

  $('#sbToggle').addEventListener('click', toggleSidebar);
  $('#sbCollapse').addEventListener('click', () => setCollapsed(true));
  $('#backBtn').addEventListener('click', goBack);
  $('#fwdBtn').addEventListener('click', goForward);

  $('#sideNav').addEventListener('click', event => {
    const device = event.target.closest('[data-device]');
    if (device) {
      openDevice(device.dataset.device);
      if (window.matchMedia('(max-width: 720px)').matches) setCollapsed(true);
      return;
    }
    const row = event.target.closest('[data-nav]');
    if (!row) return;
    const value = row.dataset.nav;
    if (value === 'devices') {
      openDevices();
      return;
    }
    if (value.startsWith('tag:')) navigate({ type: 'tag', id: value.slice(4) });
    else navigate({ type: 'nav', id: value });
    if (window.matchMedia('(max-width: 720px)').matches) setCollapsed(true);
  });

  $('#crumbs').addEventListener('click', event => {
    const target = event.target.closest('[data-goto]');
    if (!target) return;
    navigate(tokenRoute(target.dataset.goto));
  });

  $('#gridBtn').addEventListener('click', () => setView('grid'));
  $('#listBtn').addEventListener('click', () => setView('list'));

  $('#sortBtn').addEventListener('click', () => {
    const rect = $('#sortBtn').getBoundingClientRect();
    showMenu(sortMenuItems(), rect.left, rect.bottom + 7);
  });

  $('#moreBtn').addEventListener('click', () => {
    const rect = $('#moreBtn').getBoundingClientRect();
    showMenu(moreMenuItems(), rect.left, rect.bottom + 7);
  });

  $('#workspace').addEventListener('click', () => {
    const rect = $('#workspace').getBoundingClientRect();
    showMenu(workspaceMenuItems(), rect.left, rect.bottom + 4);
  });

  const searchInput = $('#searchInput');
  searchInput.addEventListener('input', () => {
    state.query = searchInput.value;
    $('#searchBox').classList.toggle('has-val', !!searchInput.value);
    state.selection.clear();
    render();
  });
  searchInput.addEventListener('keydown', event => {
    if (event.key === 'Escape') {
      event.stopPropagation();
      clearSearch(false);
      searchInput.blur();
    }
  });
  $('#searchClear').addEventListener('click', () => clearSearch(true));

  $('#viewMount').addEventListener('click', event => {
    const sort = event.target.closest('[data-sort]');
    if (sort) {
      state.sort = sort.dataset.sort;
      render();
      return;
    }

    const emptyAction = event.target.closest('[data-empty-act]');
    if (emptyAction) {
      const act = emptyAction.dataset.emptyAct;
      if (act === 'clear-search') clearSearch(false);
      else if (act === 'new-folder') newFolder();
      else if (act === 'go-drive') navigate({ type: 'nav', id: 'drive' });
      else if (act === 'mount-device') mountDevice(state.route.id);
      else navigate({ type: 'nav', id: 'pictures' });
      return;
    }

    const quick = event.target.closest('[data-act]');
    if (quick) {
      const keyEl = quick.closest('[data-key]');
      const key = keyEl ? keyEl.dataset.key : null;
      if (!key) return;
      event.stopPropagation();
      if (quick.dataset.act === 'preview') openPreview(key);
      else if (quick.dataset.act === 'open') openKey(key);
      else if (quick.dataset.act === 'menu') {
        const rect = quick.getBoundingClientRect();
        openKeyMenu(key, rect.left, rect.bottom + 6);
      }
      return;
    }

    const keyEl = event.target.closest('[data-key]');
    if (keyEl) selectKey(keyEl.dataset.key, event);
    else clearSelection();
  });

  $('#viewMount').addEventListener('dblclick', event => {
    if (event.target.closest('[data-act]')) return;
    const keyEl = event.target.closest('[data-key]');
    if (keyEl) openKey(keyEl.dataset.key);
  });

  $('#content').addEventListener('contextmenu', event => {
    event.preventDefault();
    const keyEl = event.target.closest('[data-key]');
    if (keyEl) {
      if (!state.selection.has(keyEl.dataset.key)) selectKey(keyEl.dataset.key, {});
      openKeyMenu(keyEl.dataset.key, event.clientX, event.clientY);
    } else {
      openEmptyMenu(event.clientX, event.clientY);
    }
  });

  $('#content').addEventListener('scroll', closeMenus, { passive: true });

  $('#menuLayer').addEventListener('click', event => {
    const button = event.target.closest('.mi');
    if (!button || button.disabled) return;
    const menu = button.closest('.menu');
    const item = menu && menu._items ? menu._items[Number(button.dataset.mi)] : null;
    if (!item || item.sub) return;
    const key = menu._key;
    closeMenus();
    executeMenu(item, key);
  });

  $('#menuLayer').addEventListener('pointerover', event => {
    const button = event.target.closest('.mi');
    if (!button || button.disabled) return;
    const menu = button.closest('.menu');
    if (!menu || menu.classList.contains('sub')) return;
    const item = menu._items ? menu._items[Number(button.dataset.mi)] : null;
    if (item && item.sub) openSubmenu(button, item.sub);
    else closeSubmenus();
  });

  $('#menuLayer').addEventListener('pointerleave', closeSubmenus);

  $('#pvPrev').addEventListener('click', () => stepPreview(-1));
  $('#pvNext').addEventListener('click', () => stepPreview(1));
  $('#pvOpen').addEventListener('click', () => {
    const found = findItem(state.previewKey);
    if (!found) return;
    const kind = previewKind(found.item);
    if (kind === 'app') {
      launchApp(found.item);
      return;
    }
    closeModals();
    openShare(state.previewKey, 'openwith');
  });
  $('#pvReveal').addEventListener('click', () => {
    const found = findItem(state.previewKey);
    closeModals();
    if (found && found.parent) navigate({ type: 'folder', id: found.parent.id });
    else if (!(state.route.type === 'nav' && state.route.id === 'pictures')) navigate({ type: 'nav', id: 'pictures' });
    toast(`Revealed in ${found && found.parent ? found.parent.name : 'Pictures'}`, null, null, 'folder');
  });
  $('#pvMedia').addEventListener('click', event => {
    if (event.target.closest('#pvPlay')) { toggleAudioPlayback(); return; }
    seekAudio(event);
  });

  /* ---------- get info ---------- */
  $('#infoRename').addEventListener('click', () => {
    const key = state.infoKey;
    if (!key) return;
    closeModals();
    render();
    requestAnimationFrame(() => startRename(key));
  });
  $('#infoTag').addEventListener('click', event => {
    const key = state.infoKey;
    if (!key) return;
    const rect = event.currentTarget.getBoundingClientRect();
    openTagPicker(key, rect.left, rect.bottom + 6);
  });
  $('#infoShare').addEventListener('click', () => {
    if (state.infoKey) openShare(state.infoKey, 'share');
  });

  /* ---------- share ---------- */
  $('#shareBody').addEventListener('click', event => {
    const dest = event.target.closest('[data-share-app]');
    if (dest) shareToApp(dest.dataset.shareApp);
  });
  $('#shareDone').addEventListener('click', closeModals);

  /* ---------- wallpaper ---------- */
  $('#wallGrid').addEventListener('click', event => {
    const option = event.target.closest('[data-wall]');
    if (option) setWallpaper(option.dataset.wall);
  });
  $('#wallDone').addEventListener('click', closeModals);

  /* ---------- about ---------- */
  $('#aboutLinks').addEventListener('click', event => {
    const link = event.target.closest('[data-about]');
    if (!link) return;
    const id = link.dataset.about;
    if (id === 'tokens') openModal('tokens');
    else if (id === 'shortcuts') openShortcuts();
    else if (id === 'wallpaper') openWallpaper();
    else if (id === 'devices') openDevices();
    else if (id === 'phone') openPhone();
  });

  /* ---------- shortcuts ---------- */
  $('#scList').addEventListener('click', event => {
    const row = event.target.closest('.sc-row');
    if (!row) return;
    toast(`${row.querySelector('span').textContent} — ${row.querySelector('kbd').textContent}`, null, null, 'list-check');
  });

  /* ---------- devices ---------- */
  $('#devList').addEventListener('click', event => {
    const act = event.target.closest('[data-dev-act]');
    if (!act) return;
    const id = act.dataset.device;
    const kind = act.dataset.devAct;
    if (kind === 'mount') mountDevice(id);
    else if (kind === 'eject') ejectDevice(id);
    else if (kind === 'open') openDevice(id);
  });
  $('#devDone').addEventListener('click', closeModals);

  /* ---------- transfer ---------- */
  $('#trRail').addEventListener('click', event => {
    const place = event.target.closest('[data-tr-place]');
    if (place) selectTransferPlace(place.dataset.trPlace);
  });
  $('#trList').addEventListener('click', event => {
    const dest = event.target.closest('[data-tr-dest]');
    if (!dest || !state.transfer) return;
    state.transfer.dest = dest.dataset.trDest;
    renderTransfer();
  });
  $('#trSide').addEventListener('click', event => {
    const mode = event.target.closest('[data-tr-mode]');
    if (mode && state.transfer) {
      state.transfer.mode = mode.dataset.trMode;
      state.transfer.phase = 'idle';
      renderTransfer();
      return;
    }
    if (event.target.closest('#trGo')) runTransfer(state.transfer ? state.transfer.dest : null);
  });
  $('#trCancel').addEventListener('click', closeModals);

  /* ---------- phone ---------- */
  $('#phList').addEventListener('click', event => {
    const row = event.target.closest('[data-ph-act]');
    if (row) phoneAction(row.dataset.phAct);
  });
  $('#phDone').addEventListener('click', closeModals);

  /* ---------- confirm ---------- */
  $('#cfOk').addEventListener('click', () => {
    const action = state.confirmAction;
    state.confirmAction = null;
    closeModals();
    if (action) action();
  });
  $('#cfCancel').addEventListener('click', () => {
    state.confirmAction = null;
    closeModals();
  });

  /* ---------- window chrome ---------- */
  $('#restoreBtn').addEventListener('click', restoreWindow);

  $('#copyTokens').addEventListener('click', () => copyText(TOML_SOURCE));

  $('#askBtn').addEventListener('click', toggleAI);
  $('#aiSend').addEventListener('click', () => sendAI());
  $('#aiInput').addEventListener('keydown', event => {
    if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
      event.preventDefault();
      sendAI();
    }
  });
  $('#aiInput').addEventListener('input', () => {
    const input = $('#aiInput');
    input.style.height = '38px';
    input.style.height = `${Math.min(input.scrollHeight, 96)}px`;
  });
  $('#aiPanel').addEventListener('click', event => {
    const chip = event.target.closest('[data-prompt]');
    if (chip) sendAI(chip.dataset.prompt);
  });

  $('#cmdBtn').addEventListener('click', () => openPalette());
  $('#cmdClose').addEventListener('click', () => closePalette());
  $('#paletteScrim').addEventListener('click', () => closePalette());
  $('#cmdAsk').addEventListener('click', askFromPalette);
  $('#cmdRails').addEventListener('click', event => {
    const rail = event.target.closest('[data-rail]');
    if (rail) setPaletteRail(rail.dataset.rail);
  });
  $('#cmdInput').addEventListener('input', event => {
    state.paletteQuery = event.target.value;
    state.paletteIndex = 0;
    renderPalette();
  });
  $('#cmdList').addEventListener('pointerover', event => {
    const row = event.target.closest('[data-cmd-index]');
    if (!row) return;
    const index = Number(row.dataset.cmdIndex);
    if (index !== state.paletteIndex) selectPaletteIndex(index, false);
  });
  $('#cmdList').addEventListener('click', event => {
    const empty = event.target.closest('[data-cmd-empty]');
    if (empty) {
      if (empty.dataset.cmdEmpty === 'ask') askFromPalette();
      else searchFromPalette();
      return;
    }
    const row = event.target.closest('[data-cmd-index]');
    if (!row) return;
    const index = Number(row.dataset.cmdIndex);
    selectPaletteIndex(index, false);
    runSelectedPaletteCommand();
  });
  $('#cmdDetail').addEventListener('click', event => {
    if (event.target.closest('#cmdRun')) runSelectedPaletteCommand();
  });

  document.addEventListener('keydown', event => {
    const target = event.target;
    const typing = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;
    const meta = event.metaKey || event.ctrlKey;

    if (state.paletteOpen) {
      if (event.key === 'Escape') {
        event.preventDefault();
        closePalette();
        return;
      }
      if (meta && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        closePalette();
        return;
      }
      if (event.key === 'ArrowDown') {
        event.preventDefault();
        selectPaletteIndex(state.paletteIndex + 1, true);
        return;
      }
      if (event.key === 'ArrowUp') {
        event.preventDefault();
        selectPaletteIndex(state.paletteIndex - 1, true);
        return;
      }
      if (event.key === 'Home') {
        event.preventDefault();
        selectPaletteIndex(0, true);
        return;
      }
      if (event.key === 'End') {
        event.preventDefault();
        selectPaletteIndex(state.paletteItems.length - 1, true);
        return;
      }
      if (event.key === 'Tab') {
        event.preventDefault();
        cyclePaletteRail(event.shiftKey ? -1 : 1);
        return;
      }
      if (event.key === 'Enter') {
        event.preventDefault();
        runSelectedPaletteCommand();
        return;
      }
      return;
    }

    if (event.key === 'Escape') {
      if (state.submenu) { closeSubmenus(); return; }
      if ($('#menuLayer').children.length) { closeMenus(); return; }
      if (state.openModal) { closeModals(); return; }
      if (state.aiOpen) { closeAI(); return; }
      if (typing && target.id === 'searchInput') { clearSearch(false); target.blur(); return; }
      if (state.selection.size) { clearSelection(); return; }
    }

    if (state.openModal === 'preview') {
      if (event.key === 'ArrowRight') { event.preventDefault(); stepPreview(1); return; }
      if (event.key === 'ArrowLeft') { event.preventDefault(); stepPreview(-1); return; }
    }

    if (meta && event.key.toLowerCase() === 'k') { event.preventDefault(); openPalette(); return; }
    if (meta && event.key.toLowerCase() === 'f') { event.preventDefault(); openSearch(); return; }
    if (!typing && event.key === '/') { event.preventDefault(); openSearch(); return; }
    if (meta && event.key === '\\') { event.preventDefault(); toggleSidebar(); return; }
    if (meta && event.key.toLowerCase() === 'a' && !typing) { event.preventDefault(); selectAll(); return; }
    if (meta && event.key.toLowerCase() === 'c' && !typing) { event.preventDefault(); copySelection(); return; }
    if (meta && event.key.toLowerCase() === 'x' && !typing) {
      event.preventDefault();
      const selected = singleSelected();
      if (selected) beginTransfer('move', selected);
      else toast('Select something to move', null, null, 'folder');
      return;
    }
    if (meta && event.key.toLowerCase() === 'v' && !typing) { event.preventDefault(); pasteItem(); return; }
    if (meta && event.key.toLowerCase() === 'i' && !typing) {
      event.preventDefault();
      const selected = singleSelected();
      if (selected) openInfo(selected);
      else openLocationInfo();
      return;
    }
    if (meta && event.key === '[') { event.preventDefault(); goBack(); return; }
    if (meta && event.key === ']') { event.preventDefault(); goForward(); return; }
    if (meta && event.shiftKey && event.key.toLowerCase() === 'n') { event.preventDefault(); newFolder(); return; }
    if (meta && event.key === 'Backspace') { event.preventDefault(); trashSelection(); return; }

    if (typing) return;

    const key = event.key.toLowerCase();
    if (key === 't') { openModal('tokens'); return; }
    if (key === 'g') { toggleGrid(); return; }
    if (key === 'd') { toggleTheme(); return; }
    if (event.key === '?') { openShortcuts(); return; }
    if (event.key === 'F2') {
      const selected = singleSelected();
      if (selected) startRename(selected);
      return;
    }
    if (event.key === ' ') {
      event.preventDefault();
      if (state.openModal === 'preview') closeModals();
      else {
        const selected = singleSelected();
        if (selected) openPreview(selected);
      }
      return;
    }
    if (event.key === 'Enter') {
      const selected = singleSelected();
      if (selected) openKey(selected);
    }
  });
}

/* ============================================================
   boot
   ============================================================ */

const TOML_SOURCE = "# ============================================================================\n# smart.toml \u2014 Smart Design contract\n# ----------------------------------------------------------------------------\n# This file is normative. It is the only source of colours, surfaces, shadows,\n# rims, radii, type sizes, spacing, motion, focus, and component dimensions.\n#\n# The prototype CSS token block is generated from this file:\n#   python3 smart-design/tools/generate_tokens.py\n#\n# Rules:\n#   \u2022 Never add a raw colour/shadow/radius/duration in component CSS.\n#   \u2022 Cast shadows (e1\u2026e5) and rims (rim-1\u2026rim-5) are separate. One of each per\n#     raised surface. Never stack two top highlights.\n#   \u2022 Text-bearing overlays are near-opaque. Blur is atmosphere, not legibility.\n#   \u2022 Meaningful text uses ink-1\u2026ink-3. ink-4 is decorative/disabled only.\n#   \u2022 White text is never used on the dark accent. Use ink_on_accent.\n#\n# Adapters: CSS custom properties \u00b7 gpui-kit Theme \u00b7 egui theme.\n# ============================================================================\n\n[meta]\nname = \"Smart Design\"\nversion = \"1.0.0\"\ndate = \"2026-11-02\"\nstatus = \"stable\"\nauthor = \"Smart Foundation\"\ncanonical = \"smart-design/smart.toml\"\nprototype = \"prototypes/smart-shell/index.html\"\ngallery = \"smart-design/gallery.html\"\ncanvas = \"prototypes/smart-shell/assets/wallpaper.jpg\"\nwcag = \"2.2 AA\"\n\n# ============================================================================\n# COLOUR\n# ============================================================================\n\n[color.light]\n# Ink \u2014 cool, wallpaper-derived. ink_3 is the lightest text colour allowed.\nink_1 = \"#141d30\"\nink_2 = \"#3f4b60\"\nink_3 = \"#667286\"\nink_4 = \"#8b95a6\"          # decorative/disabled only\nink_on_accent = \"#ffffff\"\n\n# Accent \u2014 cobalt. Light-mode hover DARKENS.\naccent = \"#2f6feb\"\naccent_hover = \"#2864d8\"\naccent_press = \"#2058c2\"\naccent_ink = \"#1c56c4\"\naccent_tint = \"rgba(47,111,235,0.10)\"\naccent_tint_2 = \"rgba(47,111,235,0.055)\"\naccent_tint_3 = \"rgba(47,111,235,0.18)\"\naccent_glow = \"rgba(47,111,235,0.26)\"\n\n# Semantics \u2014 base tones for fills, -ink for text on light surfaces.\ndanger = \"#c53b48\"\ndanger_ink = \"#b33440\"\ndanger_tint = \"rgba(197,59,72,0.10)\"\ngood = \"#2b9b6d\"\ngood_ink = \"#237d58\"\ngood_tint = \"rgba(43,155,109,0.10)\"\nwarn = \"#c9861a\"\nwarn_ink = \"#9c620c\"\nwarn_tint = \"rgba(201,134,26,0.11)\"\n\n# Surfaces \u2014 glass and near-opaque text planes.\nwindow_top = \"rgba(255,255,255,0.86)\"\nwindow_bottom = \"rgba(241,246,253,0.82)\"\nsidebar_top = \"rgba(222,233,248,0.78)\"\nsidebar_bottom = \"rgba(230,239,250,0.66)\"\ncontent_top = \"rgba(255,255,255,0.34)\"\ncontent_bottom = \"rgba(248,251,255,0.18)\"\npanel_top = \"rgba(252,253,255,0.995)\"\npanel_bottom = \"rgba(246,249,255,0.99)\"\nmenu_bg = \"rgba(251,253,255,0.995)\"\nfield = \"rgba(255,255,255,0.62)\"\nfield_hover = \"rgba(255,255,255,0.82)\"\ncontrol = \"rgba(255,255,255,0.68)\"\ncontrol_hover = \"rgba(255,255,255,0.88)\"\ncontrol_press = \"rgba(255,255,255,0.52)\"\nchip = \"rgba(255,255,255,0.54)\"\nrow_hover = \"rgba(24,35,56,0.055)\"\nrow_press = \"rgba(24,35,56,0.082)\"\ncard_hover = \"rgba(255,255,255,0.55)\"\nhair = \"rgba(20,29,48,0.075)\"\nhair_strong = \"rgba(20,29,48,0.13)\"\nseam = \"rgba(255,255,255,0.56)\"\nscrim = \"rgba(20,30,48,0.32)\"\ntooltip_bg = \"rgba(20,29,48,0.96)\"\ntooltip_ink = \"#f7faff\"\npalette_rail = \"rgba(24,35,56,0.035)\"\npalette_context = \"rgba(255,255,255,0.34)\"\ncode_bg = \"#0e1625\"\ncode_ink = \"#d8e2f2\"\nwall_base = \"#9aa7bb\"\nwall_image = \"assets/wallpaper.jpg\"\nwall_veil = \"linear-gradient(180deg,rgba(233,239,248,0.04),rgba(215,224,238,0.16) 60%,rgba(206,217,233,0.26))\"\n\n[color.dark]\nink_1 = \"#eef3fb\"\nink_2 = \"#c6cfdf\"\nink_3 = \"#9aa6ba\"\nink_4 = \"#737f92\"          # decorative/disabled only\nink_on_accent = \"#0a1222\"  # NEVER white on the light accent\n\naccent = \"#6f9bff\"\naccent_hover = \"#86acff\"\naccent_press = \"#5b87f0\"\naccent_ink = \"#a7c1ff\"\naccent_tint = \"rgba(111,155,255,0.155)\"\naccent_tint_2 = \"rgba(111,155,255,0.075)\"\naccent_tint_3 = \"rgba(111,155,255,0.24)\"\naccent_glow = \"rgba(111,155,255,0.32)\"\n\ndanger = \"#ff7a84\"\ndanger_ink = \"#ffa5ab\"\ndanger_tint = \"rgba(255,122,132,0.14)\"\ngood = \"#59c79a\"\ngood_ink = \"#75d8af\"\ngood_tint = \"rgba(89,199,154,0.13)\"\nwarn = \"#e5b05a\"\nwarn_ink = \"#f0c477\"\nwarn_tint = \"rgba(229,176,90,0.14)\"\n\nwindow_top = \"rgba(20,27,40,0.88)\"\nwindow_bottom = \"rgba(13,19,30,0.90)\"\nsidebar_top = \"rgba(13,19,30,0.78)\"\nsidebar_bottom = \"rgba(10,15,25,0.68)\"\ncontent_top = \"rgba(255,255,255,0.022)\"\ncontent_bottom = \"rgba(255,255,255,0.006)\"\npanel_top = \"rgba(24,32,47,0.995)\"\npanel_bottom = \"rgba(15,22,34,0.995)\"\nmenu_bg = \"rgba(27,35,50,0.995)\"\nfield = \"rgba(255,255,255,0.065)\"\nfield_hover = \"rgba(255,255,255,0.10)\"\ncontrol = \"rgba(255,255,255,0.072)\"\ncontrol_hover = \"rgba(255,255,255,0.115)\"\ncontrol_press = \"rgba(255,255,255,0.05)\"\nchip = \"rgba(255,255,255,0.06)\"\nrow_hover = \"rgba(255,255,255,0.055)\"\nrow_press = \"rgba(255,255,255,0.085)\"\ncard_hover = \"rgba(255,255,255,0.055)\"\nhair = \"rgba(255,255,255,0.085)\"\nhair_strong = \"rgba(255,255,255,0.15)\"\nseam = \"rgba(255,255,255,0.10)\"\nscrim = \"rgba(3,7,14,0.58)\"\ntooltip_bg = \"rgba(238,243,251,0.96)\"\ntooltip_ink = \"#141d30\"\npalette_rail = \"rgba(255,255,255,0.035)\"\npalette_context = \"rgba(255,255,255,0.035)\"\ncode_bg = \"#080e19\"\ncode_ink = \"#d8e2f2\"\nwall_base = \"#0b1220\"\nwall_image = \"assets/wallpaper.jpg\"\nwall_veil = \"linear-gradient(180deg,rgba(5,10,19,0.52),rgba(4,9,17,0.66))\"\n\n[color.contrast]\nmin_text = 4.5\nmin_large = 3.0\nmin_ui = 3.0\non_accent_min = 4.5\nink_4_usage = \"decorative or disabled only\"\n\n# ============================================================================\n# TYPE\n# ============================================================================\n\n[type]\nfamily = \"InterVariable\"\nfallback = [\"SF Pro Text\", \"Segoe UI\", \"system-ui\", \"sans-serif\"]\nmono = [\"SF Mono\", \"ui-monospace\", \"JetBrains Mono\", \"Cascadia Code\", \"Menlo\", \"monospace\"]\nfeatures = [\"cv05\", \"cv08\", \"ss03\"]\nsize_label = \"10.5px\"\nsize_meta = \"11.5px\"\nsize_body = \"13px\"\nsize_title = \"15px\"\nsize_page = \"26px\"\nsize_keycap = \"10px\"\nsize_micro = \"10px\"\nheight_body = 1.45\nheight_title = 1.25\nweight_body = 510\nweight_medium = 565\nweight_semibold = 610\nweight_bold = 645\ntracking_tight = \"-0.012em\"\ntracking_title = \"-0.014em\"\ntracking_page = \"-0.024em\"\ntracking_label = \"0.075em\"\n\n# ============================================================================\n# SPACE + SHAPE\n# ============================================================================\n\n[space]\nunit = 4\ngap_xs = 4\ngap_sm = 8\ngap_md = 12\ngap_lg = 16\ngap_xl = 24\ngap_2xl = 32\nsidebar_width = 252\nsidebar_collapsed = 72\nappbar_height = 60\nstatus_height = 34\ncontent_pad_x = 26\ncontent_pad_x_md = 20\ncontent_pad_x_sm = 16\nrow_height = 32\nmenu_row_height = 33\ntouch_target = 44\ndialog_gutter = 20\n\n[shape]\nwindow = 24\npanel = 22\nmenu = 20\ncard = 16\ntile = 13\nitem = 11\nfield = 999\npill = 999\nkeycap = 4\nfocus = 2\n\n# ============================================================================\n# ELEVATION \u2014 cast shadow and rim are separate. Use exactly one of each.\n# ============================================================================\n\n[elevation.light]\nmodel = \"cast + rim\"\ne0 = \"none\"\ne1 = \"0 1px 2px rgba(16,26,44,0.07)\"\ne2 = \"0 3px 10px -3px rgba(16,26,44,0.13), 0 1px 2px rgba(16,26,44,0.05)\"\ne3 = \"0 14px 34px -12px rgba(16,26,44,0.24), 0 3px 10px -2px rgba(16,26,44,0.08)\"\ne4 = \"0 28px 72px -16px rgba(14,24,42,0.40), 0 6px 18px -6px rgba(14,24,42,0.16)\"\ne5 = \"0 54px 140px -28px rgba(9,19,37,0.55), 0 18px 48px -18px rgba(9,19,37,0.30)\"\nrim_1 = \"inset 0 1px 0 rgba(255,255,255,0.62), inset 0 -1px 0 rgba(16,26,44,0.04)\"\nrim_2 = \"inset 0 1px 0 rgba(255,255,255,0.70), inset 0 -1px 0 rgba(16,26,44,0.05)\"\nrim_3 = \"inset 0 1px 0 rgba(255,255,255,0.64), inset 0 -1px 0 rgba(16,26,44,0.06)\"\nrim_4 = \"inset 0 1px 0 rgba(255,255,255,0.72), inset 0 -1px 0 rgba(16,26,44,0.08)\"\nrim_5 = \"inset 0 1px 0 rgba(255,255,255,0.74), inset 0 -1px 0 rgba(16,26,44,0.10)\"\nrim_accent = \"inset 0 1px 0 rgba(255,255,255,0.22), inset 0 -1px 0 rgba(0,0,0,0.12)\"\nrim_inset = \"inset 0 1px 2px rgba(16,26,44,0.08)\"\nrim_well = \"inset 0 2px 6px rgba(16,26,44,0.12), inset 0 0 0 1px rgba(16,26,44,0.05)\"\n\n[elevation.dark]\nmodel = \"cast + rim\"\ne0 = \"none\"\ne1 = \"0 1px 2px rgba(0,0,0,0.28)\"\ne2 = \"0 3px 10px -3px rgba(0,0,0,0.40), 0 1px 2px rgba(0,0,0,0.24)\"\ne3 = \"0 16px 38px -12px rgba(0,0,0,0.58), 0 3px 10px -2px rgba(0,0,0,0.30)\"\ne4 = \"0 32px 80px -16px rgba(0,0,0,0.70), 0 8px 22px -6px rgba(0,0,0,0.44)\"\ne5 = \"0 60px 150px -30px rgba(0,0,0,0.86), 0 20px 54px -18px rgba(0,0,0,0.60)\"\nrim_1 = \"inset 0 1px 0 rgba(255,255,255,0.055), inset 0 -1px 0 rgba(0,0,0,0.18)\"\nrim_2 = \"inset 0 1px 0 rgba(255,255,255,0.065), inset 0 -1px 0 rgba(0,0,0,0.20)\"\nrim_3 = \"inset 0 1px 0 rgba(255,255,255,0.07), inset 0 -1px 0 rgba(0,0,0,0.22)\"\nrim_4 = \"inset 0 1px 0 rgba(255,255,255,0.08), inset 0 -1px 0 rgba(0,0,0,0.24)\"\nrim_5 = \"inset 0 1px 0 rgba(255,255,255,0.09), inset 0 -1px 0 rgba(0,0,0,0.26)\"\nrim_accent = \"inset 0 1px 0 rgba(255,255,255,0.30), inset 0 -1px 0 rgba(0,0,0,0.14)\"\nrim_inset = \"inset 0 1px 2px rgba(0,0,0,0.30)\"\nrim_well = \"inset 0 2px 8px rgba(0,0,0,0.34), inset 0 0 0 1px rgba(255,255,255,0.055)\"\n\n# ============================================================================\n# MOTION\n# ============================================================================\n\n[motion]\nease_out = \"cubic-bezier(0.22,1,0.36,1)\"\nease_in_out = \"cubic-bezier(0.65,0,0.35,1)\"\nspring = \"cubic-bezier(0.34,1.42,0.44,1)\"\nfast = \"140ms\"\nbase = \"220ms\"\nslow = \"360ms\"\nscene = \"520ms\"\nmenu = \"220ms\"\nsidebar = \"360ms\"\ntransfer_prepare = \"300ms\"\ntransfer_min = \"1600ms\"\ntransfer_land = \"420ms\"\ntransfer_verify = \"350ms\"\nreduced = \"fade only\"\n\n# ============================================================================\n# FOCUS\n# ============================================================================\n\n[focus]\nstyle = \"ring + halo\"\nwidth = 2\noffset = 2\nhalo = \"0 0 0 3px\"\nhalo_color = \"rgba(47,111,235,0.22)\"\ndark_halo_color = \"rgba(111,155,255,0.26)\"\nvisible = \"always\"\n\n# ============================================================================\n# ICONS\n# ============================================================================\n\n[icon]\nnavigation = 20\nidentity = 20\naction = 16\naction_lg = 18\nstatus = 14\nclose_dialog = 18\nclose_media = 20\nrail = 20\nrow = 16\nquick_action = 15\nempty = 30\n\n# ============================================================================\n# CHROME + LAYOUT\n# ============================================================================\n\n[chrome]\npattern = \"unified\"\nsidebar = \"full-height\"\ntraffic_lights = \"sidebar-top-left\"\ntraffic_lights_collapsed = \"appbar-leading\"\ncaption_buttons = \"traffic-lights\"\nappbar_background = \"transparent\"\nappbar_controls = \"tonal-elevated\"\nsidebar_edge = \"tonal-seam\"\nwindow_drag = [\"sidebar-top\", \"appbar\"]\nsearch_shortcut = \"/\"\nfind_shortcut = \"Cmd+F\"\ncommand_palette_shortcut = \"Cmd+K\"\nsidebar_shortcut = \"Cmd+\\\\\"\nai_affordance = \"Ask Smart pill\"\n\n[layout]\ngrid_columns = 4\ngrid_columns_compact = 3\ngrid_columns_narrow = 2\nfolder_columns = 4\ncard_gap = 16\nresponsive_auto_collapse = 900\nstatusbar = true\nmax_dialog_height = \"calc(100dvh - 48px)\"\nsheet_gutter = 12\n\n# ============================================================================\n# COMPONENTS \u2014 dimensions only; visual states are in COMPONENTS.md.\n# ============================================================================\n\n[component.control]\nheight_xs = 26\nheight_sm = 30\nheight_md = 34\nheight_lg = 40\nheight_xl = 44\nradius = 999\ngap = 7\nicon = 16\nicon_lg = 18\nshine = \"rim only\"\n\n[component.appbar]\nheight = 60\nbutton = \"pill\"\nbutton_size = 34\nsearch_width = 214\nsearch_shape = \"pill\"\nsegmented = true\ntraffic_inset = 15\n\n[component.sidebar]\nwidth = 252\ncollapsed = 72\nrow_height = 32\nrow_radius = 10\nsection_label_size = \"10.5px\"\nsection_label_tracking = \"0.075em\"\nstorage_card = true\nfooter_caption = true\n\n[component.card]\nmedia_ratio = \"16 / 10\"\nradius = 14\nquick_action = 30\nquick_gap = 8\nquick_inset = 8\ncheck = 30\ncheck_visual = 22\nhover_lift = \"2px\"\nselection = \"2px accent ring + 2px offset + tint\"\nfocus = \"2px neutral ring + 3px offset\"\nicon_states = \"outline \u2192 duotone \u2192 filled\"\n\n[component.menu]\nwidth = 252\nsubmenu_width = 214\nradius = 18\nitem_height = 33\nitem_radius = 9\npadding = 7\ngroup_gap = 8\nstrokes = false\nbackdrop = \"none\"\nsurface = \"near-opaque tonal plate\"\ndestructive = \"danger tint\"\n\n[component.dialog]\nsm = 420\nmd = 560\nlg = 820\nxl = 1000\npadding = 20\nclose = 32\nclose_icon = 18\nmedia_close = 36\nmedia_close_icon = 20\norb = 38\norb_icon = 20\nradius = 22\n\n[component.palette]\nwidth = 880\nmax_height = 620\nsearch_height = 46\nsearch_radius = 999\nsearch_pad = 16\nrail_width = 76\nrail_icon = 20\nrail_label = \"10.5px\"\nrow_height = 44\nrow_icon_tile = 34\ncontext_width = 300\nfooter_height = 38\nrow_accent = \"3px left bar + tint + filled icon\"\n\n[component.selection]\ncheck_visual = 22\ncheck_target = 30\nring = 2\nring_offset = 2\nfocus_ring = 2\nfocus_offset = 3\nmarquee_radius = 4\nshelf_height = 56\nshelf_radius = 20\nshelf_gap = 8\n\n[component.transfer]\nwidth = 880\nmin_height = 500\nsummary_width = 296\nslab_width = 96\nwell = 88\nring = 76\nring_stroke = 4\narc_stroke = 1.5\npacket = 6\ndemo_min = \"2800ms\"\nproduction_min = \"900ms\"\ncommit = \"on landing\"\nauto_close = false\n\n[component.phone]\nwidth = 820\nstage_width = 300\nstage_height = 420\ndeck_width = 320\nradius = 22\nbeam_duration = \"520ms\"\ndevice_name_repeated = false\nrelay_destination = \"current folder\"\n\n[component.devices]\nwidth = 560\nrow_orb = 38\nrow_pad = 16\nmeter = 4\ncards = false\nmount_then_open = true\n\n# ============================================================================\n# STATES\n# ============================================================================\n\n[states]\nicon_navigation = \"duotone \u2192 filled active\"\nicon_identity = \"filled always\"\nicon_action = \"outline \u2192 duotone hover \u2192 filled active\"\nicon_status = \"filled/duotone, no swap\"\nicon_close = \"filled, larger, no swap\"\nselection = \"persistent accent tint + ring + counter\"\nhover = \"tone only; raised controls may rise one level\"\nfocus = \"2px accent ring + 3px halo\"\npress = \"scale 0.98 + inner shade\"\ndisabled = \"45% opacity + reason\"\nmenu = \"tonal plate + spaced groups + keyboard\"\n\n[laws]\nlaws = [\n  \"One light: every raised surface uses one cast shadow and one rim.\",\n  \"Depth is a promise: raised surfaces are interactive; rows stay flat.\",\n  \"One accent signal per decision.\",\n  \"Selection persists, is countable, and is never confused with hover or focus.\",\n  \"Text-bearing overlays are near-opaque; blur is not legibility.\",\n  \"Text floors: 10.5px label, 11.5px meta, 13px body.\",\n  \"Icons are roles: navigation duotone, identity/close filled, actions outline\u2192duotone\u2192filled.\",\n  \"Springs only for spatial changes; reduced motion is designed.\",\n  \"No Tab hijacking; modal focus is trapped and restored.\",\n  \"A transfer never commits before the visual lands, and never auto-closes.\",\n  \"When in doubt, remove rather than add another border, shadow, or badge.\",\n]\n";

function init() {
  FOLDERS.forEach(folder => {
    folder.items.forEach(item => { item.parentKey = folder.key; });
  });

  hydrateIcons(document);
  bindEvents();
  $('#askBtn').setAttribute('aria-expanded', 'false');
  $('#cmdBtn').setAttribute('aria-expanded', 'false');
  applyHash();

  const mq = window.matchMedia('(max-width: 900px)');
  if (mq.matches) setCollapsed(true);
  if (location.hash.includes('drawer')) setCollapsed(false);
  const onChange = event => setCollapsed(event.matches);
  if (mq.addEventListener) mq.addEventListener('change', onChange);
  else if (mq.addListener) mq.addListener(onChange);
}

document.addEventListener('DOMContentLoaded', init);

})();
