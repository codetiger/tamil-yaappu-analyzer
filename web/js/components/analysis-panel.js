/**
 * analysis-panel.js — Renders classification badge + compact tag grid.
 * Tags are clickable — detail is shown in the sidebar via onTagClick callback.
 */

// ===== Labels & Maps =====

const TAG_LABELS = {
  etukai: 'Etukai (எதுகை)',
  monai: 'Monai (மோனை)',
  iyaipu: 'Iyaipu (இயைபு)',
  kutrilugaram: 'Kutrilugaram (குற்றியலுகரம்)',
  valid_tamil: 'Valid Tamil',
  no_empty_words: 'No Empty Words',
  syllabification_ok: 'Syllabification OK',
  has_overflow: 'Overflow Seer',
  has_compound_words: 'Compound Words',
  thalai_all_valid: 'All Junctions Valid',
  sol_per_adi: 'Words/Line (சொல்/அடி)',
  seer_pattern: 'Seer Pattern (சீர்)',
  thalai_types: 'Junction Types (தளை)',
  eetru_type: 'Eetru Type (ஈற்று)',
};

const TAG_SHORT_LABELS = {
  etukai: 'எதுகை',
  monai: 'மோனை',
  iyaipu: 'இயைபு',
  kutrilugaram: 'குற்றியலுகரம்',
  valid_tamil: 'Valid',
  no_empty_words: 'No Empty',
  syllabification_ok: 'Syllables',
  has_overflow: 'Overflow',
  has_compound_words: 'Compound',
  thalai_all_valid: 'Junctions',
  sol_per_adi: 'Words/Line',
  seer_pattern: 'Seer',
  thalai_types: 'Thalai',
  eetru_type: 'Eetru',
};

const CLASSIFICATION_LABELS = {
  venba: 'வெண்பா (Venba)',
  venba_possible: 'வெண்பா? (Venba - possible)',
  unknown: 'Unknown',
  kural_venba: 'குறள் வெண்பா (Kural Venba)',
  sindhiyal_venba: 'சிந்தியல் வெண்பா (Sindhiyal Venba)',
  alaviyal_venba: 'அளவு வெண்பா (Alaviyal Venba)',
  pahrodai_venba: 'பஃறொடை வெண்பா (Pahrodai Venba)',
  kali_venba: 'கலி வெண்பா (Kali Venba)',
};

const SEER_TAMIL = {
  thema: 'தேமா', pulima: 'புளிமா', koovilam: 'கூவிளம்', karuvilam: 'கருவிளம்',
  themangai: 'தேமாங்காய்', themangani: 'தேமாங்கனி',
  koovilankai: 'கூவிளங்காய்', koovilankani: 'கூவிளங்கனி',
  pulimangai: 'புளிமாங்காய்', pulimangani: 'புளிமாங்கனி',
  karuvilangai: 'கருவிளங்காய்', karuvilankani: 'கருவிளங்கனி',
  overflow: 'மிகை',
};

const THALAI_TAMIL = {
  iyarseer_vendalai: 'இயற்சீர் வெண்டளை',
  venseer_vendalai: 'வெண்சீர் வெண்டளை',
  cross_category: 'குறுக்கு',
  intra_compound: 'கூட்டுச்சொல்',
};

// ===== Tag Groups =====

const TAG_GROUPS = [
  { label: 'Ornamentation (அணி)', keys: ['etukai', 'monai', 'iyaipu'] },
  { label: 'Structure (அமைப்பு)', keys: ['valid_tamil', 'no_empty_words', 'sol_per_adi'] },
  { label: 'Meter (சீர்)', keys: ['kutrilugaram', 'syllabification_ok', 'has_overflow', 'has_compound_words', 'eetru_type', 'seer_pattern'] },
  { label: 'Junctions (தளை)', keys: ['thalai_all_valid', 'thalai_types'] },
];

// Tags where true = negative (show as gray when true)
const NEGATIVE_TAGS = new Set(['has_overflow', 'has_compound_words']);

// ===== Main Render =====

/**
 * Render classification badge + compact clickable tag grid.
 * @param {HTMLElement} container
 * @param {object|null} analysis
 * @param {function} onTagClick - callback(tagKey, tags) when a tag is clicked
 */
export function renderAnalysis(container, analysis, onTagClick) {
  container.innerHTML = '';
  if (!analysis) return;

  container.classList.add('visible');

  if (analysis.classification) {
    renderClassification(container, analysis.classification);
  }

  if (analysis.tags) {
    renderTagGrid(container, analysis.tags, onTagClick);
  }
}

function renderClassification(container, classification) {
  const section = document.createElement('div');
  section.className = 'classification-section';

  const venbaType = classification.venba_type || 'unknown';
  const paaFamily = classification.paa_family || 'unknown';

  const badge = document.createElement('div');
  badge.className = `classification-badge ${venbaType !== 'unknown' ? 'classified' : 'unclassified'}`;
  const label = CLASSIFICATION_LABELS[venbaType] || CLASSIFICATION_LABELS[paaFamily] || venbaType;
  badge.innerHTML = `<span class="classification-label">${label}</span>`;
  section.appendChild(badge);

  const info = document.createElement('div');
  info.className = 'classification-info';
  info.textContent = `${classification.adi_count || '?'} lines, ${classification.sol_count || '?'} words`;
  section.appendChild(info);

  container.appendChild(section);
}

function renderTagGrid(container, tags, onTagClick) {
  const wrapper = document.createElement('div');
  wrapper.className = 'tag-grid-wrapper';

  const toggleBtn = document.createElement('button');
  toggleBtn.className = 'tag-toggle-btn';
  toggleBtn.innerHTML = 'Show analysis tags <span class="toggle-chevron">\u203A</span>';
  wrapper.appendChild(toggleBtn);

  const grid = document.createElement('div');
  grid.className = 'tag-grid collapsed';

  TAG_GROUPS.forEach(group => {
    const groupEl = document.createElement('div');
    groupEl.className = 'tag-group';

    const groupLabel = document.createElement('div');
    groupLabel.className = 'tag-group-label';
    groupLabel.textContent = group.label;
    groupEl.appendChild(groupLabel);

    const items = document.createElement('div');
    items.className = 'tag-group-items';

    group.keys.forEach(key => {
      if (tags[key] === undefined) return;
      const item = createTagItem(key, tags, onTagClick);
      items.appendChild(item);
    });

    groupEl.appendChild(items);
    grid.appendChild(groupEl);
  });

  toggleBtn.addEventListener('click', () => {
    const isExpanded = grid.classList.contains('expanded');
    if (isExpanded) {
      grid.classList.remove('expanded');
      grid.classList.add('collapsed');
      toggleBtn.classList.remove('expanded');
      toggleBtn.innerHTML = 'Show analysis tags <span class="toggle-chevron">\u203A</span>';
    } else {
      grid.classList.remove('collapsed');
      grid.classList.add('expanded');
      toggleBtn.classList.add('expanded');
      toggleBtn.innerHTML = 'Hide analysis tags <span class="toggle-chevron">\u203A</span>';
    }
  });

  wrapper.appendChild(grid);
  container.appendChild(wrapper);
}

function createTagItem(key, tags, onTagClick) {
  const value = tags[key];
  const item = document.createElement('div');
  item.className = 'tag-item';
  item.setAttribute('role', 'button');
  item.setAttribute('tabindex', '0');

  // Determine status
  let status;
  if (typeof value === 'boolean') {
    const isNeg = NEGATIVE_TAGS.has(key);
    status = (isNeg ? !value : value) ? 'pass' : 'fail';
  } else {
    status = 'info';
  }
  item.classList.add(`tag-${status}`);

  const dot = document.createElement('span');
  dot.className = 'tag-dot';
  item.appendChild(dot);

  const label = document.createElement('span');
  label.className = 'tag-item-label';
  label.textContent = TAG_SHORT_LABELS[key] || key;
  item.appendChild(label);

  // Brief inline value for non-boolean tags
  if (typeof value === 'string') {
    const val = document.createElement('span');
    val.className = 'tag-item-brief';
    val.textContent = value;
    item.appendChild(val);
  } else if (Array.isArray(value) && key === 'sol_per_adi') {
    const val = document.createElement('span');
    val.className = 'tag-item-brief';
    val.textContent = value.join('+');
    item.appendChild(val);
  }

  const detail = tags[key + '_detail'];
  if (detail || Array.isArray(value)) {
    item.classList.add('has-detail');
  }

  item.addEventListener('click', () => onTagClick(key, tags));
  item.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onTagClick(key, tags);
    }
  });

  return item;
}

// ===== Tag Detail (rendered in sidebar) =====

/**
 * Render tag detail in the detail panel (sidebar / bottom sheet).
 * @param {HTMLElement} container - the detail-zone element
 * @param {string} tagKey
 * @param {object} tags - all tags
 */
export function renderTagDetail(container, tagKey, tags) {
  container.innerHTML = '';
  container.classList.add('visible');

  // Drag handle
  const dragHandle = document.createElement('div');
  dragHandle.className = 'detail-drag-handle';
  container.appendChild(dragHandle);

  const panel = document.createElement('div');
  panel.className = 'detail-panel';

  // Header
  const header = document.createElement('div');
  header.className = 'detail-header';

  const headerContent = document.createElement('div');
  headerContent.className = 'detail-header-content';

  const title = document.createElement('div');
  title.className = 'detail-word-text';
  title.textContent = TAG_LABELS[tagKey] || tagKey;
  headerContent.appendChild(title);

  const subtitle = document.createElement('div');
  subtitle.className = 'detail-seer-info';
  subtitle.textContent = getTagGroupLabel(tagKey);
  headerContent.appendChild(subtitle);

  header.appendChild(headerContent);

  const closeBtn = document.createElement('button');
  closeBtn.className = 'detail-close-btn';
  closeBtn.setAttribute('aria-label', 'Close detail panel');
  closeBtn.innerHTML = '&#215;';
  closeBtn.addEventListener('click', () => {
    container.dispatchEvent(new CustomEvent('detail-close'));
  });
  header.appendChild(closeBtn);

  panel.appendChild(header);

  // Body
  const body = document.createElement('div');
  body.className = 'detail-body';

  const value = tags[tagKey];

  // Value section
  renderTagValue(body, tagKey, value);

  // Detail string
  const detail = tags[tagKey + '_detail'];
  if (detail) {
    renderDetailString(body, detail);
  }

  // Array breakdown
  if (Array.isArray(value) && tagKey !== 'sol_per_adi') {
    renderArrayBreakdown(body, tagKey, value);
  }

  panel.appendChild(body);
  container.appendChild(panel);
}

function getTagGroupLabel(key) {
  for (const g of TAG_GROUPS) {
    if (g.keys.includes(key)) return g.label;
  }
  return '';
}

function renderTagValue(body, key, value) {
  const section = document.createElement('div');
  section.className = 'tag-detail-value-section';

  if (typeof value === 'boolean') {
    const isNeg = NEGATIVE_TAGS.has(key);
    const pass = isNeg ? !value : value;

    const badge = document.createElement('div');
    badge.className = `tag-detail-badge ${pass ? 'pass' : 'fail'}`;
    badge.textContent = value ? 'Yes' : 'No';
    section.appendChild(badge);
  } else if (typeof value === 'string') {
    const badge = document.createElement('div');
    badge.className = 'tag-detail-badge info';
    badge.textContent = value;
    section.appendChild(badge);
  } else if (Array.isArray(value)) {
    if (key === 'sol_per_adi') {
      const badge = document.createElement('div');
      badge.className = 'tag-detail-badge info';
      badge.textContent = value.join(' + ');
      section.appendChild(badge);

      // Per-line breakdown
      value.forEach((count, i) => {
        const line = document.createElement('div');
        line.className = 'tag-detail-line';
        line.textContent = `Line ${i + 1}: ${count} words`;
        section.appendChild(line);
      });
    }
  }

  body.appendChild(section);
}

function renderDetailString(body, detail) {
  const section = document.createElement('div');
  section.className = 'breakdown-section';

  const title = document.createElement('div');
  title.className = 'breakdown-section-title';
  title.textContent = 'Detail';
  section.appendChild(title);

  const text = document.createElement('div');
  text.className = 'tag-detail-text';
  text.textContent = detail;
  section.appendChild(text);

  body.appendChild(section);
}

function renderArrayBreakdown(body, key, values) {
  const section = document.createElement('div');
  section.className = 'breakdown-section';

  const title = document.createElement('div');
  title.className = 'breakdown-section-title';
  title.textContent = key === 'seer_pattern' ? 'Seer Sequence' : 'Junction Sequence';
  section.appendChild(title);

  const tamilMap = key === 'seer_pattern' ? SEER_TAMIL : THALAI_TAMIL;

  const list = document.createElement('div');
  list.className = 'tag-detail-array';

  values.forEach((v, i) => {
    const row = document.createElement('div');
    row.className = 'tag-detail-array-item';

    const idx = document.createElement('span');
    idx.className = 'tag-detail-array-idx';
    idx.textContent = `${i + 1}`;
    row.appendChild(idx);

    const tamil = tamilMap[v] || '';
    const labelText = tamil ? `${tamil}` : v;
    const lbl = document.createElement('span');
    lbl.className = 'tag-detail-array-label';
    lbl.textContent = labelText;
    row.appendChild(lbl);

    if (tamil) {
      const eng = document.createElement('span');
      eng.className = 'tag-detail-array-eng';
      eng.textContent = v;
      row.appendChild(eng);
    }

    list.appendChild(row);
  });

  section.appendChild(list);
  body.appendChild(section);
}
